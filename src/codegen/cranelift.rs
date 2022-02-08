use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, FuncId, Linkage, Module, ModuleError};

use crate::{
    compiler::{
        fcf::FlatExpr,
        ssa::{Instr, SSA},
    },
    parser::ast::{Lit, LitKind},
};

/// Basic JIT structure (from the cranelift demo)
pub struct JIT {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext,
    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,
    /// The data context, which is to data objects what `ctx` is to functions.
    data_ctx: DataContext,
    /// The module, with the jit backend, which manages the JIT'd
    /// functions.
    module: JITModule,
    /// Calling functions in Cranelift require local declerations, hence why we need
    /// to keep track of mappings of function names to function ids.
    funcs: HashMap<String, FuncId>,
}

impl JIT {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
            funcs: HashMap::new(),
        }
    }

    /// Generate machine code given the input SSA.
    pub fn codegen(mut self, ssa: SSA) -> Result<*const u8, ModuleError> {
        for bind in ssa.binds {
            match bind.fexpr {
                FlatExpr::Lit(lit) => {
                    // NOTE: At the time of writing, I have no idea what this does :^)
                    self.data_ctx.define(lit.into());
                    let id = self
                        .module
                        .declare_data(&bind.name, Linkage::Export, true, false)?;

                    self.module.define_data(id, &self.data_ctx)?;
                    self.data_ctx.clear();
                    self.module.finalize_definitions();
                }
                _ => unimplemented!("only literal expressions are allowed at the top level."),
            }
        }

        let int = types::I64;
        for proc in ssa.procs {
            // FIXME: We only have one type: U64.
            self.ctx.func.signature.params.push(AbiParam::new(int));
            self.ctx.func.signature.returns.push(AbiParam::new(int));
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
            let entry_block = builder.create_block();
            // Since this is the entry block, add block parameters corresponding to
            // the function's parameters.
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            // Tell the builder that this block will have no further
            // predecessors. Since it's the entry block, it won't have any
            // predecessors.
            builder.seal_block(entry_block);
            // Declare parameter variable.
            let param_val = builder.block_params(entry_block)[0];
            let param_var = Variable::new(0);
            builder.declare_var(param_var, int);
            builder.def_var(param_var, param_val);
            // Declare return variable.
            let mut translator = Translator {
                builder,
                module: &mut self.module,
                funcs: &self.funcs,
            };
            let Instr::Local(v, e) = &proc.body.last().unwrap().instrs.first().unwrap();
            let return_var = Variable::new(*v);
            let return_val = translator.fexpr_to_value(&*e);
            translator.builder.declare_var(return_var, int);
            translator.builder.def_var(return_var, return_val);
            // Emit the return instruction.
            translator.builder.ins().return_(&[return_val]);
            // Tell the builder we're done with this function.
            translator.builder.finalize();
            // Next, declare the function to jit. Functions must be declared
            // before they can be called, or defined.
            let id = self.module.declare_function(
                &proc.name,
                Linkage::Export,
                &self.ctx.func.signature,
            )?;
            // Define the function to jit. This finishes compilation, although
            // there may be outstanding relocations to perform. Currently, jit
            // cannot finish relocations until all functions to be called are
            // defined. For this toy demo for now, we'll just finalize the
            // function below.
            self.module.define_function(
                id,
                &mut self.ctx,
                &mut codegen::binemit::NullTrapSink {},
                &mut codegen::binemit::NullStackMapSink {},
            )?;
            // Now that compilation is finished, we can clear out the context state.
            println!("{} = {}", &proc.name, &self.ctx.func);
            self.module.clear_context(&mut self.ctx);
            // Finalize the functions which we just defined, which resolves any
            // outstanding relocations (patching in addresses, now that they're
            // available).
            self.module.finalize_definitions();
            // Insert the FuncId alongside it's real name into the funcs map to
            // resolve function calls.
            self.funcs.insert(proc.name.clone(), id);
        }
        // We can now retrieve a pointer to the machine code, starting from main.
        let code = self
            .module
            .get_finalized_function(*self.funcs.get("main").unwrap());
        Ok(code)
    }
}

impl From<Lit> for Box<[u8]> {
    fn from(lit: Lit) -> Self {
        match lit.kind {
            LitKind::Int(int) => Box::new(int.to_le_bytes()),
        }
    }
}

struct Translator<'t> {
    builder: FunctionBuilder<'t>,
    module: &'t mut JITModule,
    funcs: &'t HashMap<String, FuncId>,
}

impl<'t> Translator<'t> {
    fn fexpr_to_value(&mut self, fexpr: &FlatExpr) -> Value {
        match fexpr {
            FlatExpr::Lit(lit) => {
                let LitKind::Int(number) = lit.kind;
                self.builder.ins().iconst(types::I64, number)
            }
            // FIXME: Assume that the used variable is the parameter.
            FlatExpr::Var(_) => self.builder.use_var(Variable::new(0)),
            FlatExpr::Call(fname, fexpr) => {
                let caller = *self.funcs.get(fname).unwrap();
                let callee = self
                    .module
                    .declare_func_in_func(caller, &mut self.builder.func);
                let args = &[self.fexpr_to_value(&*fexpr)];
                let call = self.builder.ins().call(callee, args);
                self.builder.inst_results(call)[0]
            }
        }
    }
}

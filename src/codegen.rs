use crate::compiler::fcf::{Expr, FlatExpr, Func, FCF};
use crate::mlir::attribute::{Attribute, NamedAttribute};
use crate::mlir::block::Block;
use crate::mlir::operation::{self, Operation};
use crate::mlir::region::Region;
use crate::mlir::{Context, Module, Pass};
use crate::parser::ast::Lit;

pub struct Generator {
    ctx: Context,
    src: FCF,
}

impl Generator {
    pub fn new(source: FCF) -> Self {
        Generator {
            ctx: Context::new(),
            src: source,
        }
    }

    pub fn translate_lit(&self, lit: &Lit) -> Operation {
        let location = self.ctx.get_unknown_location();
        let u64_type = self.ctx.get_uint_type(64);
        let Lit::Int(int) = lit;
        let lit_value =
            NamedAttribute::new(&self.ctx, "value", Attribute::new_integer(u64_type, *int));
        operation::Builder::new("arith.constant", location)
            .attributes(&[lit_value])
            .results(&[u64_type])
            .build()
    }

    pub fn translate_fexpr(&self, fexpr: &FlatExpr) -> Region {
        let location = self.ctx.get_unknown_location();
        let u64_type = self.ctx.get_uint_type(64);
        let FlatExpr { expr, .. } = fexpr;
        let expr_op = match expr {
            Expr::Lit(lit) => self.translate_lit(lit),
            _ => unimplemented!(),
        };
        let ret = operation::Builder::new("std.return", location)
            .operands(&[expr_op.get_res(0)])
            .build();
        let mut block = Block::new(&[(u64_type, location)]);
        block.append(expr_op);
        block.append(ret);
        let mut region = Region::new();
        region.append(block);
        region
    }

    pub fn translate_func(&self, func: &Func) -> Operation {
        let location = self.ctx.get_unknown_location();
        let u64_type = self.ctx.get_uint_type(64);
        let func_name = NamedAttribute::new(
            &self.ctx,
            "sym_name",
            Attribute::new_string(&self.ctx, &func.name),
        );
        let region = self.translate_fexpr(&func.fexpr);
        let filler_type = self.ctx.get_func_type(&[u64_type], &[u64_type]);
        let func_type = NamedAttribute::new(&self.ctx, "type", Attribute::new_type(filler_type));
        operation::Builder::new("builtin.func", location)
            .attributes(&[func_name, func_type])
            .regions(vec![region])
            .build()
    }

    pub fn run(self) {
        let location = self.ctx.get_unknown_location();
        let mut module = Module::new(location);
        for func in &self.src.funcs {
            let func_op = self.translate_func(func);
            module.append(func_op);
        }
        let main_op = self.translate_func(&self.src.main);
        module.append(main_op);
        Pass::new(&self.ctx).std_to_llvm().run(&mut module);
        print!("{}", Operation::from(module))
    }
}


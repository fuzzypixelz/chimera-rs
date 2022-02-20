pub mod raw {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod attribute;
pub mod block;
pub mod operation;
pub mod region;
pub mod types;
pub mod value;

use std::mem::ManuallyDrop;

use operation::Operation;
use raw::*;

impl From<&str> for MlirStringRef {
    fn from(item: &str) -> Self {
        MlirStringRef {
            data: item.as_ptr() as *const _,
            length: item.len() as u64,
        }
    }
}

/// Wrapper around the C API's MlirContext.
pub struct Context {
    context: MlirContext,
}

impl Context {
    /// Make an empty MLIR context.
    ///
    /// Currently, this also registers all dialects and all passes for your convenience;
    /// which is not particularly efficient and is subject to change.
    pub fn new() -> Self {
        unsafe {
            let context = mlirContextCreate();
            mlirRegisterAllDialects(context);
            mlirRegisterAllPasses();
            Context { context }
        }
    }

    /// Make a source location from a `filename`, a `line` number and a `column` number.
    ///
    /// The object is created in, and owned by the context.
    pub fn get_location(&self, filename: &str, line: usize, column: usize) -> Location {
        Location {
            location: unsafe {
                mlirLocationFileLineColGet(
                    self.as_raw(),
                    filename.into(),
                    line as u32,
                    column as u32,
                )
            },
        }
    }

    /// Make an unknown source location.
    ///
    /// The object is created in, and owned by the context.
    pub fn get_unknown_location(&self) -> Location {
        Location {
            location: unsafe { mlirLocationUnknownGet(self.as_raw()) },
        }
    }

    /// Get a Context from a raw MlirContext.
    pub fn from_raw(context: MlirContext) -> Self {
        Context { context }
    }

    /// Return the underlying raw MlirAttribute.
    pub fn as_raw(&self) -> MlirContext {
        self.context
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { mlirContextDestroy(self.context) }
    }
}

/// Wrapper around the C API's MlirModule.
pub struct Module {
    module: MlirModule,
}

impl Module {
    /// Make an empty MLIR Module from a source location.
    pub fn new(location: Location) -> Self {
        Module {
            module: unsafe { mlirModuleCreateEmpty(location.into_raw()) },
        }
    }

    /// Append an `operation` to the module's only body block.
    ///
    /// We make the opinionated choice of only exposing the block
    /// this way for now.
    pub fn append(&mut self, operation: Operation) {
        unsafe {
            mlirBlockAppendOwnedOperation(mlirModuleGetBody(self.module), operation.into_raw())
        }
    }

    /// Get a Context from a raw MlirModule.
    pub fn from_raw(module: MlirModule) -> Self {
        Module { module }
    }

    /// Return the underlying raw MlirModule.
    pub fn as_raw(&self) -> MlirModule {
        self.module
    }

    /// Return the underlying raw MlirModule and consume the Module.
    pub fn into_raw(self) -> MlirModule {
        ManuallyDrop::new(self).module
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { mlirModuleDestroy(self.module) }
    }
}

#[derive(Clone, Copy)]
/// Wrapper around the C API's MlirLocation.
pub struct Location {
    location: MlirLocation,
}

impl Location {
    /// Unwrap the Location, returning the underlying MlirLocation.
    fn into_raw(self) -> MlirLocation {
        self.location
    }
}

/// Wrapper around the C API's MlirPassManager.
pub struct Pass {
    pass: MlirPassManager,
}

impl Pass {
    /// Make an empty MLIR pass.
    ///
    /// See the dialect_to_dialect() methods for available conversions.
    pub fn new(context: &Context) -> Self {
        Pass {
            pass: unsafe { mlirPassManagerCreate(context.as_raw()) },
        }
    }

    /// Standard to LLVM conversion pass.
    pub fn std_to_llvm(self) -> Self {
        unsafe {
            let conversion = mlirCreateConversionConvertStandardToLLVM();
            mlirPassManagerAddOwnedPass(self.pass, conversion);
        }
        self
    }

    /// SCF to OpenMP conversion pass.
    pub fn scf_to_openmp(self) -> Self {
        unsafe {
            let conversion = mlirCreateConversionConvertSCFToOpenMP();
            mlirPassManagerAddOwnedPass(self.pass, conversion);
        }
        self
    }

    /// OpenMP to LLVM conversion pass.
    pub fn openmp_to_llvm(self) -> Self {
        unsafe {
            let conversion = mlirCreateConversionConvertOpenMPToLLVM();
            mlirPassManagerAddOwnedPass(self.pass, conversion);
        }
        self
    }

    /// Run the pass on a specified module.
    ///
    /// Doesn't consume the pass so you can reuse it on other multiple modules.
    pub fn run(&self, module: &mut Module) {
        // TODO: Do proper error handling with the LogicalResult.
        unsafe {
            mlirPassManagerRun(self.pass, module.as_raw());
        }
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        unsafe { mlirPassManagerDestroy(self.pass) }
    }
}

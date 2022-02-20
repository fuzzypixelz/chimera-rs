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

/// Wrapper around the C API's MlirContext since we can't implement Drop for Copy types.
pub struct Context {
    context: MlirContext,
}

impl Context {
    /// Make an empty MLIR context.
    ///
    /// Currently, this also registers all dialects and all passes for your convenience; which is not particularly efficient and is subject to change.
    pub fn new() -> Self {
        unsafe {
            let context = mlirContextCreate();
            mlirRegisterAllDialects(context);
            mlirRegisterAllPasses();
            Context { context }
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

/// Wrapper around the C API's MlirModule since we can't implement Drop for Copy types.
pub struct Module {
    module: MlirModule,
}

impl Module {
    /// Make an empty MLIR Module from a source location.
    pub fn new(location: MlirLocation) -> Self {
        Module {
            module: unsafe { mlirModuleCreateEmpty(location) },
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

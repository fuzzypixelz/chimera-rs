use super::*;
use std::mem::ManuallyDrop;
use std::slice;

/// Construct an MLIR Operation from a set of attributes, results and operands.
pub struct Builder {
    state: MlirOperationState,
}

impl Builder {
    /// Make a new MLIR Operation builder from the operation's name (with the namespace prefix)
    /// and its location in the source code.
    pub fn new(name: &str, loc: MlirLocation) -> Self {
        Builder {
            state: unsafe { mlirOperationStateGet(name.into(), loc) },
        }
    }

    /// Enable type inference for the constructed Operation, results should not
    /// be added when enabling this.
    pub fn infer_results(mut self) -> Self {
        unsafe { mlirOperationStateEnableResultTypeInference(&mut self.state) }
        self
    }

    /// Add named attributes to the constructed Operation.
    pub fn attributes(mut self, items: &[MlirNamedAttribute]) -> Self {
        unsafe {
            mlirOperationStateAddAttributes(&mut self.state, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add operands to the constructed Operation.
    pub fn operands(mut self, items: &[MlirValue]) -> Self {
        unsafe {
            mlirOperationStateAddOperands(&mut self.state, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Finalize construction of the operation and consume the builer.
    pub fn build(self) -> Operation {
        // We cannot drop the state in this case since its ownership is
        // transferred to MlirOperation.
        let mut builder = ManuallyDrop::new(self);
        Operation {
            operation: unsafe { mlirOperationCreate(&mut builder.state) },
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        // MlirOperationState owns the regions it points to but nothing else.
        let regions =
            unsafe { slice::from_raw_parts(self.state.regions, self.state.nRegions as usize) };
        for &region in regions {
            unsafe {
                mlirRegionDestroy(region);
            }
        }
    }
}

/// Wrapper around the C API's MlirOperation since we can't implement Drop for Copy types.
pub struct Operation {
    operation: MlirOperation,
}

impl Operation {
    pub unsafe fn as_raw(&self) -> MlirOperation {
        self.operation
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        // unsafe { mlirOperationDestroy(self.operation) }
    }
}

use super::attribute::NamedAttribute;
use super::raw::*;
use super::region::Region;
use super::value::Value;
use super::Module;
use std::mem::ManuallyDrop;
use std::slice;

/// Construct an MLIR Operation from a set of attributes, results and operands.
pub struct Builder {
    state: MlirOperationState,
}

impl Builder {
    /// Make a new MLIR Operation builder from the operation's name (with the namespace prefix)
    /// and its location in the source code.
    pub fn new(name: &str, location: MlirLocation) -> Self {
        Builder {
            state: unsafe { mlirOperationStateGet(name.into(), location) },
        }
    }

    /// Enable type inference for the constructed Operation, results should not
    /// be added when enabling this.
    pub fn infer_results(mut self) -> Self {
        unsafe { mlirOperationStateEnableResultTypeInference(&mut self.state) }
        self
    }

    /// Add named attributes to the constructed Operation.
    pub fn attributes(mut self, items: &[NamedAttribute]) -> Self {
        let items: Vec<_> = items.iter().map(NamedAttribute::as_raw).collect();
        unsafe {
            mlirOperationStateAddAttributes(&mut self.state, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add operands to the constructed Operation.
    pub fn operands(mut self, items: &[Value]) -> Self {
        let items: Vec<_> = items.iter().map(Value::as_raw).collect();
        unsafe {
            mlirOperationStateAddOperands(&mut self.state, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add regions to the constructed Operation.
    ///
    /// This takes a Boxed slice instead of a slice reference since
    /// it takes ownership of the regions.
    pub fn regions(mut self, items: Box<[Region]>) -> Self {
        unsafe {
            let size = items.len();
            let items = Box::into_raw(items);
            mlirOperationStateAddOwnedRegions(&mut self.state, size as isize, items as *const _)
        }
        self
    }

    /// Add result types to the constructed Operation.
    pub fn results(mut self, items: &[MlirType]) -> Self {
        unsafe {
            mlirOperationStateAddResults(&mut self.state, items.len() as isize, items.as_ptr())
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
        unsafe {
            // MlirOperationState owns the regions it points to but nothing else.
            let regions = slice::from_raw_parts(self.state.regions, self.state.nRegions as usize);
            for &region in regions {
                mlirRegionDestroy(region)
            }
        }
    }
}

/// Wrapper around the C API's MlirOperation since we can't implement Drop for Copy types.
pub struct Operation {
    operation: MlirOperation,
}

impl Operation {
    /// Returns the `position`-th result of the Operation.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    pub fn get_res(&self, position: usize) -> Value {
        unsafe {
            if position > self.res_len() {
                panic!("block argument position is out of bounds.")
            } else {
                let value = mlirOperationGetResult(self.operation, position as isize);
                Value::from_raw(value)
            }
        }
    }

    /// Returns the number of arguments in the Block.
    pub fn res_len(&self) -> usize {
        unsafe { mlirOperationGetNumResults(self.operation) as usize }
    }

    /// Return the underlying raw MlirOperation.
    pub fn as_raw(&self) -> MlirOperation {
        self.operation
    }

    /// Print out the operation to stderr.
    pub fn dump(&self) {
        unsafe { mlirOperationDump(self.operation) }
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        unsafe { mlirOperationDestroy(self.operation) }
    }
}

impl From<Module> for Operation {
    fn from(item: Module) -> Self {
        // While viewing the module as an operation we should drop it
        // as an Operation and avoid calling Module's Drop.
        let item = ManuallyDrop::new(item);
        Operation {
            operation: unsafe { mlirModuleGetOperation(item.as_raw()) },
        }
    }
}

use super::attribute::NamedAttribute;
use super::raw::*;
use super::region::Region;
use super::types::Type;
use super::value::Value;
use super::{Location, Module};

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::slice;
use std::str;

/// Construct an MLIR Operation from a set of attributes, results and operands.
pub struct Builder {
    inner: MlirOperationState,
}

impl Builder {
    /// Make a new MLIR Operation builder from the operation's name (with the namespace prefix)
    /// and its location in the source code.
    ///
    /// # Safety
    ///
    /// Currently it is undefined behaviour to try to provide missing attributes for an operation.
    /// For example, if one define a `builtin.func` operation without giving the mandatory "sym_name"
    /// attribute, MLIR will simply throw a Segmentation Fault. We cannot hope to provide logic for
    /// all the (growing) list of possible dialects. So in conclusion, RTFM!
    pub fn new(name: &str, location: Location<'_>) -> Self {
        Builder {
            inner: unsafe { mlirOperationStateGet(name.into(), location.into_raw()) },
        }
    }

    /// Enable type inference for the constructed Operation, results should not
    /// be added when enabling this.
    pub fn infer_results(mut self) -> Self {
        unsafe { mlirOperationStateEnableResultTypeInference(&mut self.inner) }
        self
    }

    /// Add named attributes to the constructed Operation.
    pub fn attributes(mut self, items: &[NamedAttribute<'_>]) -> Self {
        let items = items
            .iter()
            .map(|named_attr| named_attr.as_raw())
            .collect::<Box<[_]>>();
        unsafe {
            mlirOperationStateAddAttributes(&mut self.inner, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add operands to the constructed Operation.
    pub fn operands(mut self, items: &[Value<'_>]) -> Self {
        let items = items
            .iter()
            .map(|value| value.as_raw())
            .collect::<Box<[_]>>();
        unsafe {
            mlirOperationStateAddOperands(&mut self.inner, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add regions to the constructed Operation.
    ///
    /// This takes a vector instead of a slice reference since
    /// it takes ownership of the regions.
    pub fn regions(mut self, items: Vec<Region>) -> Self {
        let items = items
            .into_iter()
            .map(|region| region.into_raw())
            .collect::<Box<[_]>>();
        unsafe {
            mlirOperationStateAddOwnedRegions(&mut self.inner, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Add result types to the constructed Operation.
    pub fn results(mut self, items: &[Type<'_>]) -> Self {
        let items = items
            .into_iter()
            .map(|type_| type_.as_raw())
            .collect::<Box<[_]>>();
        unsafe {
            mlirOperationStateAddResults(&mut self.inner, items.len() as isize, items.as_ptr())
        }
        self
    }

    /// Finalize construction of the operation and consume the builer.
    pub fn build(self) -> Operation {
        // We cannot drop the state in this case since its ownership is
        // transferred to MlirOperation.
        Operation {
            inner: unsafe { mlirOperationCreate(&mut self.into_raw()) },
        }
    }

    /// Return the underlying raw MlirOperationState and consume the OperationState.
    pub fn into_raw(self) -> MlirOperationState {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            // MlirOperationState owns the regions it points to but nothing else.
            let regions = slice::from_raw_parts(self.inner.regions, self.inner.nRegions as usize);
            for &region in regions {
                mlirRegionDestroy(region)
            }
        }
    }
}

/// Wrapper around the C API's MlirOperation since we can't implement Drop for Copy types.
pub struct Operation {
    inner: MlirOperation,
}

impl Operation {
    /// Returns the `position`-th result of the Operation.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    pub fn get_res(&self, position: usize) -> Value<'_> {
        unsafe {
            if position > self.res_len() {
                panic!("block argument position is out of bounds.")
            } else {
                Value::from_raw(
                    mlirOperationGetResult(self.inner, position as isize),
                    PhantomData,
                )
            }
        }
    }

    /// Returns the number of arguments in the Block.
    pub fn res_len(&self) -> usize {
        unsafe { mlirOperationGetNumResults(self.inner) as usize }
    }

    /// Print out the operation to stderr.
    pub fn dump(&self) {
        unsafe { mlirOperationDump(self.inner) }
    }

    /// Return the underlying raw MlirOperation.
    pub fn as_raw(&self) -> MlirOperation {
        self.inner
    }

    /// Return the underlying raw MlirOperation and consume the Operation.
    pub fn into_raw(self) -> MlirOperation {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        unsafe { mlirOperationDestroy(self.inner) }
    }
}

impl From<Module> for Operation {
    fn from(item: Module) -> Self {
        // While viewing the module as an operation we should drop it
        // as an Operation and avoid calling Module's Drop.
        Operation {
            inner: unsafe { mlirModuleGetOperation(item.into_raw()) },
        }
    }
}

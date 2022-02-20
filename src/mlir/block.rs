use super::operation::Operation;
use super::raw::*;
use super::value::Value;
use std::mem::ManuallyDrop;

/// Wrapper around the C API's MlirBlock since we can't implement Drop for Copy types.
pub struct Block {
    block: MlirBlock,
}

impl Block {
    /// Make an MLIR Block from the types of its `arguments` and their corresponding source locations.
    ///
    /// This method takes a slice of pairs rather than a pair of slices to
    /// enforce the fact that the number of types must match the number of locations.
    pub fn new(arguments: &[(MlirType, MlirLocation)]) -> Self {
        let (types, locs): (Vec<_>, Vec<_>) = arguments.iter().cloned().unzip();
        Block {
            block: unsafe {
                mlirBlockCreate(arguments.len() as isize, types.as_ptr(), locs.as_ptr())
            },
        }
    }

    /// Append the `operation` at then end of the Block.
    pub fn append(&mut self, operation: Operation) {
        // Here the MlirBlock takes ownership of the Operation, so consider it dropped!
        unsafe { mlirBlockAppendOwnedOperation(self.block, operation.into_raw()) }
    }

    /// Returns the `position`-th argument of the Block.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of bounds.
    pub fn get_arg(&self, position: usize) -> Value {
        unsafe {
            if position > self.arg_len() {
                panic!("block argument position is out of bounds.")
            } else {
                let value = mlirBlockGetArgument(self.block, position as isize);
                Value::from_raw(value)
            }
        }
    }

    /// Returns the number of arguments in the Block.
    pub fn arg_len(&self) -> usize {
        unsafe { mlirBlockGetNumArguments(self.block) as usize }
    }

    /// Return the underlying raw MlirBlock.
    pub fn as_raw(&self) -> MlirBlock {
        self.block
    }

    /// Return the underlying raw MlirBlock and consume the Block.
    pub fn into_raw(self) -> MlirBlock {
        ManuallyDrop::new(self).block
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe { mlirBlockDestroy(self.block) }
    }
}

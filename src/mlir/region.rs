use std::mem::ManuallyDrop;

use super::block::Block;
use super::raw::*;

/// Wrapper around the C API's MlirRegion since we can't implement Drop for Copy types.
pub struct Region {
    inner: MlirRegion,
}

impl Region {
    /// Create an empty Region.
    pub fn new() -> Self {
        Region {
            inner: unsafe { mlirRegionCreate() },
        }
    }

    /// Append the `operation` at then end of the Block.
    pub fn append(&mut self, block: Block) {
        // Here the MlirBlock takes ownership of the Block, so consider it dropped!
        unsafe { mlirRegionAppendOwnedBlock(self.inner, block.into_raw()) }
    }

    /// Return the underlying raw MlirRegion.
    pub fn as_raw(&self) -> MlirRegion {
        self.inner
    }

    /// Return the underlying raw MlirRegion and consume the region.
    pub fn into_raw(self) -> MlirRegion {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe { mlirRegionDestroy(self.inner) }
    }
}

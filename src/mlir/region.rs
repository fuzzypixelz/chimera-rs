use std::mem::ManuallyDrop;

use super::block::Block;
use super::raw::*;

/// Wrapper around the C API's MlirRegion since we can't implement Drop for Copy types.
pub struct Region {
    region: MlirRegion,
}

impl Region {
    /// Create an empty Region.
    pub fn new() -> Self {
        Region {
            region: unsafe { mlirRegionCreate() },
        }
    }

    /// Append the `operation` at then end of the Block.
    pub fn append(&mut self, block: Block) {
        // Here the MlirBlock takes ownership of the Block, so consider it dropped!
        unsafe { mlirRegionAppendOwnedBlock(self.region, block.into_raw()) }
    }

    /// Get a Region from a raw MlirValue.
    pub fn from_raw(region: MlirRegion) -> Self {
        Region { region }
    }

    /// Return the underlying raw MlirRegion.
    pub fn as_raw(&self) -> MlirRegion {
        self.region
    }

    /// Return the underlying raw MlirRegion and consume the region.
    pub fn into_raw(self) -> MlirRegion {
        ManuallyDrop::new(self).region
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe { mlirRegionDestroy(self.region) }
    }
}

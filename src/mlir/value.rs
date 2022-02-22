use std::marker::PhantomData;

use super::raw::MlirValue;

/// Wrapper around the C API's MlirValue.
#[derive(Clone, Copy)]
pub struct Value<'v> {
    /// Opaque pointer the data across the FFI, generally a C++ object.
    inner: MlirValue,
    /// Force the type to "own" a reference to the context it was created in,
    /// so that its lifetime may be the same as that of the context.
    _marker: PhantomData<&'v ()>,
}

impl<'v> Value<'v> {
    /// Create a wrapped Value from a raw pointer and a marker representing
    /// the object (block, operation, ...) that owns this value.
    ///
    /// # Safety
    ///
    /// There is no way to tell if the marker really has the lifetime of its
    /// owning object, so it should only be used in locations where we're sure
    /// the lifetimes match and properly make our reference dependent on the owner.
    pub unsafe fn from_raw(inner: MlirValue, _marker: PhantomData<&'v ()>) -> Self {
        Value { inner, _marker }
    }

    /// Return the underlying raw MlirValue.
    pub fn as_raw(&self) -> MlirValue {
        self.inner
    }

    /// Return the underlying raw MlirValue, and consume the Value.
    pub fn into_raw(self) -> MlirValue {
        self.inner
    }
}

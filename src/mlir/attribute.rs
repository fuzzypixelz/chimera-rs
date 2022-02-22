use std::marker::PhantomData;

use super::raw::*;
use super::types::Type;
use super::Context;

#[derive(Copy, Clone)]
/// Wrapper around the C API's MlirNamedAttribute.
pub struct NamedAttribute<'n> {
    /// Opaque pointer the data across the FFI, generally a C++ object.
    inner: MlirNamedAttribute,
    /// Force the type to "own" a reference to the context it was created in,
    /// so that its lifetime may be the same as that of the context.
    _marker: PhantomData<&'n ()>,
}

impl NamedAttribute<'_> {
    /// Create an MLIR named attribute from an attribute and its name.
    pub fn new(ctx: &Context, name: &str, attr: Attribute<'_>) -> Self {
        NamedAttribute {
            inner: unsafe {
                mlirNamedAttributeGet(mlirIdentifierGet(ctx.as_raw(), name.into()), attr.as_raw())
            },
            _marker: PhantomData,
        }
    }

    /// Return the underlying raw MlirNamedAttribute.
    pub fn as_raw(&self) -> MlirNamedAttribute {
        self.inner
    }

    /// Return the underlying raw MlirNamedAttribute and consume the named attribute.
    pub fn into_raw(self) -> MlirNamedAttribute {
        self.inner
    }
}

#[derive(Copy, Clone)]
/// Wrapper around the C API's MlirAttribute.
pub struct Attribute<'a> {
    /// Opaque pointer the data across the FFI, generally a C++ object.
    inner: MlirAttribute,
    /// Force the type to "own" a reference to the context it was created in,
    /// so that its lifetime may be the same as that of the context.
    _marker: PhantomData<&'a ()>,
}

impl Attribute<'_> {
    /// Create a type attribute from a Type.
    pub fn new_type(type_: Type<'_>) -> Self {
        Attribute {
            inner: unsafe { mlirTypeAttrGet(type_.into_raw()) },
            _marker: PhantomData,
        }
    }

    /// Create a string attribute from a str.
    pub fn new_string(ctx: &Context, string: &str) -> Self {
        Attribute {
            inner: unsafe { mlirStringAttrGet(ctx.as_raw(), string.into()) },
            _marker: PhantomData,
        }
    }

    /// Create an signed integer attribute from a type and a value.
    pub fn new_integer(type_: Type<'_>, value: i64) -> Self {
        Attribute {
            inner: unsafe { mlirIntegerAttrGet(type_.into_raw(), value) },
            _marker: PhantomData,
        }
    }

    /// Return the underlying raw MlirAttribute.
    pub fn as_raw(&self) -> MlirAttribute {
        self.inner
    }

    /// Return the underlying raw MlirAttribute, and consume the attribute.
    pub fn into_raw(self) -> MlirAttribute {
        self.inner
    }
}

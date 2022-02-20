use super::types::Type;
use super::{raw::*, Context};

#[derive(Copy, Clone)]
/// Wrapper around the C API's MlirNamedAttribute.
pub struct NamedAttribute {
    named_attr: MlirNamedAttribute,
}

impl NamedAttribute {
    /// Create an MLIR named attribute from an attribute and its name.
    pub fn new(ctx: &Context, name: &str, attr: Attribute) -> Self {
        NamedAttribute {
            named_attr: unsafe {
                mlirNamedAttributeGet(mlirIdentifierGet(ctx.as_raw(), name.into()), attr.as_raw())
            },
        }
    }

    /// Get an Attribute from a raw MlirAttribute.
    pub fn from_raw(named_attr: MlirNamedAttribute) -> Self {
        NamedAttribute { named_attr }
    }

    /// Return the underlying raw MlirAttribute.
    pub fn as_raw(&self) -> MlirNamedAttribute {
        self.named_attr
    }
}

#[derive(Copy, Clone)]
/// Wrapper around the C API's MlirAttribute.
pub struct Attribute {
    attr: MlirAttribute,
}

impl Attribute {
    /// Create a type attribute from a Type.
    pub fn new_type(type_: Type) -> Self {
        Attribute {
            attr: unsafe { mlirTypeAttrGet(type_.into_raw()) },
        }
    }

    /// Create a string attribute from a str.
    pub fn new_string(ctx: &Context, string: &str) -> Self {
        Attribute {
            attr: unsafe { mlirStringAttrGet(ctx.as_raw(), string.into()) },
        }
    }

    /// Create an integer attribute from a type and a size.
    pub fn new_integer(type_: Type, size: usize) -> Self {
        Attribute {
            attr: unsafe { mlirIntegerAttrGet(type_.into_raw(), size as i64) },
        }
    }

    /// Get a Value from a raw MlirValue.
    pub fn from_raw(attr: MlirAttribute) -> Self {
        Attribute { attr }
    }

    /// Return the underlying raw MlirAttribute.
    pub fn as_raw(&self) -> MlirAttribute {
        self.attr
    }
}

use super::raw::MlirValue;

/// Wrapper around the C API's MlirValue.
#[derive(Clone, Copy)]
pub struct Value {
    value: MlirValue,
}

impl Value {
    /// Get a Value from a raw MlirValue.
    pub fn from_raw(value: MlirValue) -> Self {
        Value { value }
    }

    /// Return the underlying raw MlirValue.
    pub fn as_raw(&self) -> MlirValue {
        self.value
    }
}

use super::{raw::*, Context};

#[derive(Clone, Copy)]
/// Wrapper around the C API's MlirType.
pub struct Type {
    inner: MlirType,
}

impl Type {
    /// Dump the Type into stderr.
    pub fn dump(self) {
        unsafe { mlirTypeDump(self.into_raw()) }
    }

    /// Unwrap the Type, returning the underlying MlirType.
    pub fn into_raw(self) -> MlirType {
        self.inner
    }

    /// Unwrap the Type, returning the underlying MlirType without moving.
    pub fn as_raw(&self) -> MlirType {
        self.inner
    }
}

impl Context {
    /// Make an MLIR unsigned integer type of specified bit `width`.
    pub fn get_uint_type(&self, width: u32) -> Type {
        Type {
            inner: unsafe { mlirIntegerTypeGet(self.as_raw(), width) },
        }
    }

    /// Make an MLIR signed integer type of specified bit `width`.
    pub fn get_int_type(&self, width: u32) -> Type {
        Type {
            inner: unsafe { mlirIntegerTypeSignedGet(self.as_raw(), width) },
        }
    }

    pub fn get_func_type(&self, input: &[Type], result: &[Type]) -> Type {
        let input = input
            .iter()
            .cloned()
            .map(Type::into_raw)
            .collect::<Box<[_]>>();
        let result = result
            .iter()
            .cloned()
            .map(Type::into_raw)
            .collect::<Box<[_]>>();
        Type {
            inner: unsafe {
                mlirFunctionTypeGet(
                    self.as_raw(),
                    input.len() as isize,
                    input.as_ptr(),
                    result.len() as isize,
                    result.as_ptr(),
                )
            },
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        unsafe { mlirTypeEqual(self.into_raw(), other.into_raw()) }
    }
}

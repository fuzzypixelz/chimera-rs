use std::{fmt, marker::PhantomData};

use super::{raw::*, stringref_printer_callback, Context};

#[derive(Clone, Copy)]
/// Wrapper around the C API's MlirType.
pub struct Type<'t> {
    /// Opaque pointer the data across the FFI, generally a C++ object.
    inner: MlirType,
    /// Force the type to "own" a reference to the context it was created in,
    /// so that its lifetime may be the same as that of the context.
    _marker: PhantomData<&'t ()>,
}

impl Type<'_> {
    /// Dump the Type into stderr.
    pub fn dump(&self) {
        unsafe { mlirTypeDump(self.as_raw()) }
        // NOTE: The above function doesn't seem to add a newline.
        eprintln!()
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
    pub fn get_uint_type(&self, width: u32) -> Type<'_> {
        Type {
            inner: unsafe { mlirIntegerTypeGet(self.as_raw(), width) },
            _marker: PhantomData,
        }
    }

    /// Make an MLIR signed integer type of specified bit `width`.
    pub fn get_int_type(&self, width: u32) -> Type<'_> {
        Type {
            inner: unsafe { mlirIntegerTypeSignedGet(self.as_raw(), width) },
            _marker: PhantomData,
        }
    }

    pub fn get_func_type(&self, input: &[Type<'_>], result: &[Type<'_>]) -> Type<'_> {
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
            _marker: PhantomData,
        }
    }
}

impl PartialEq for Type<'_> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { mlirTypeEqual(self.into_raw(), other.into_raw()) }
    }
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            mlirTypePrint(
                self.inner,
                Some(stringref_printer_callback),
                f as *mut _ as *mut _,
            )
        }
        Ok(())
    }
}

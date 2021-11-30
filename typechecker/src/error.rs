use thiserror::Error;

use common::Expr;

use crate::Type;

#[derive(Error, Debug)]
pub enum TypeError {
    // Attempt to apply a function of arity 0
    #[error("Function takes no arguments")]
    ZeroArgFuncApplication { expr: Expr },
    // The function was supplied an argument of
    // the wrong type.
    #[error("Function was supplied a wrong type")]
    ArgOfWrongType { expr: Expr, expected: Type, found: Type },
    // User referenced an inexistent type, or something
    // which is not a type.
    #[error("Invalid type idnetifier")]
    InvalidTypeName,
    // Number of function params doesn't match type.
    #[error("Invalid parameters and/or types in function.")]
    TypeAnnotationMismatch,
    // At this point, the user should always explicitly specify
    // the return expression.
    #[error("Function has no return value")]
    NoReturnValueFound,
    #[error("Return value isn't of the annotated type")]
    InvalidReturnType
}

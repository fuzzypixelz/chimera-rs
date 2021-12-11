use thiserror::Error;

use crate::ast::Expr;

use crate::typechecker::Type;

#[derive(Error, Debug)]
pub enum TypeError {
    // Attempt to apply a function of arity 0
    #[error("the function expression `{left:?}` takes no arguments \
            but was supplied `{right:?}.`")]
    ApplicationOfZeroArgFunc { left: Expr, right: Expr },
    // The function was supplied an argument of
    // the wrong type.
    #[error("the function expression `{expr:?}` expects arguments of type `{expected:?}` \
            but was supplied a `{found:?}`.")]
    ApplicationOnWrongType { expr: Expr, expected: Type, found: Type },
    // User referenced an inexistent type, or something
    // which is not a type.
    #[error("the name `{name}` doesn't correspond to a Type.")]
    InvalidTypeName { name: String },
    #[error("the name `{name}` doesn't correspond to any let-defintion.")]
    InvalidName { name: String },
    // Mismatched types
    #[error("the function declaration of `{name}` specifies an `{expected:?}` as a return type,\n\
             but the compiler found a `{found:?}` in its retuned expression.")]
    InvalidReturnType { name: String, expected: Type, found: Type },
    // Number of function params doesn't match type.
    // #[error("the function declaration of `{name}` has {found:?} params,\nwhile its type annotation \
    //         suggests it should have {expected:?} params.")]
    // FunctionParamsConflictWithType { name: String, expected: usize, found: usize },
    #[error("the variable `{name}` is of type {expected:?} but was assigned to a {found:?}.")]
    AssignmentOfWrongType { name: String, expected: Type, found: Type },
    #[error("the expression `{cond:?}` is of type `{found:?}` but was used as a branch condition.")]
    BranchConditionOfWrongType { cond: Expr, found: Type },
}

use polytype::UnificationError;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum LexicalError {
    #[error("Invalid syntax")]
    InvalidSyntax,
}

#[derive(Error, Debug, PartialEq)]
pub enum TypeError {
    // FIXME: do proper error reporting, this would require:
    //  1. translating this error to a human-readable version
    //  2. recording information about where in the source code
    //     the expression shows and use annotate-snippets.
    #[error("unification error")]
    UnificationError(#[from] UnificationError),
    #[error("the name `{0}` is not in scope")]
    ScopeError(String),
}

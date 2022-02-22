use crate::parser::ast::{Lit, Node};
use polytype::Type;

/// Concrete Syntax Tree, all expressions are type annotated.
#[derive(Debug)]
pub struct CST {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Definition { name: String, texpr: TypedExpr },
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub kind: Type,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct TypedParam {
    pub kind: Type,
    pub param: String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Void,
    Lit(Lit),
    Name(String),
    Lambda {
        tparams: Vec<TypedParam>,
        texpr: Box<TypedExpr>,
    },
    /// Curried Call: the number of arguments is less
    /// than the lambda's arity. This will be handled
    /// by generating a continuation in the form of a
    /// function which takes the remaining arguments.
    Apply {
        tleft: Box<TypedExpr>,
        tright: Vec<TypedExpr>,
    },
    /// Saturated Call: the number of arguments matches
    /// the lambda's arity, which means this maps directly
    /// to a fast C-like function call.
    Call {
        tleft: Box<TypedExpr>,
        tright: Vec<TypedExpr>,
    },
    Block {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Item(Item),
    Expr(TypedExpr),
}

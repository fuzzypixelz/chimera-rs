use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Primitive data types
    // handled by the implementation.
    Void,
    I64,
    Bool,
    String,
    // Type declared by the user, uses
    // information from the AST.
    New(DType),
    // Type for single-argument, single-
    // result functions: T -> T
    Pure(Box<Type>, Box<Type>),
    Impure(Box<Type>, Box<Type>),
    // Type of a variable, can be any of
    // the above. The "return type" of a
    // Pure should never be this.
    Var(Box<Type>),
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub decls: HashMap<String, Decl>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Type(DType),
    Func(DFunc),
    // Actor(Actor),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DType {
    // Type decleration. Consists of the new Type's name,
    // and the list of its constructros.
    name: String,
    body: Vec<(String, Vec<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DFunc {
    pub sig: Type,
    pub name: String,
    pub body: Vec<Instr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Sig {
    ty: Type,
    params: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    // This is meant to force evaluation of a certain Expr,
    // for example for the result at a function's end. Or,
    // for "applying" a function purely for its side-effects.
    Compute(Expr),
    // Create a name in the local environement that maps to a
    // mutable value. An Assign Instr allows this change.
    // NOTE: Obviously this is only allowed in impure functions.
    Bind(Var),
    Assign(Assign),
    // The toplevel of a program contains function and type
    // declarations. The same is allowed in the body of a function.
    Decl(Decl),
    // This is necessary for special Instr's such as break for control
    // flow in simple `loop ... end` blocks.
    Keyword(Keyword),
    Branch(Branch),
    Loop(Loop),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Prim(Prim),
    Name(String),
    Apply { name: String, args: Vec<Expr> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Prim {
    Void,
    I64(i64),
    // U64(u64),
    // F64(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Apply {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub id: String,
    pub ty: String,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Branch {
    pub paths: Vec<(Expr, Vec<Instr>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    pub body: Vec<Instr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Break,
    Ellipsis,
    // Instrinsic(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub name: String,
    pub expr: Expr,
}

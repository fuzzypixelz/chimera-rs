use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct AST {
    pub decls: Vec<Decl>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Func(DFunc),
    Type(DType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DType {
    // Type decleration. Consists of the new Type's name,
    // and the list of its constructros.
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<DConstr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DConstr {
    pub name: String,
    pub record: Option<Vec<(String, Ann)>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DFunc {
    pub name: String,
    pub params: Params,
    pub ann: Ann,
    pub op: AssignOP,
    pub body: Vec<Instr>,
}

// pub type Name   = String;
pub type Ann = Vec<String>;
pub type Params = Vec<String>;

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    // This is meant to force evaluation of a certain Expr,
    // for example for the result at a function's end. Or,
    // for "applying" a function purely for its side-effects.
    Compute(Expr),
    // Create a name in the local environement that maps to a
    // mutable value. An Assign Instr allows this change.
    // NOTE: Obviously this is only allowed in impure functions.
    Bind {
        op: AssignOP,
        name: String,
        expr: Expr,
        ann: Ann,
    },
    Assign {
        op: AssignOP,
        name: String,
        expr: Expr,
    },
    // The toname: Stringplevel of a program contains function and type
    // declarexpr: Expr, ations. The same is allowed in the body of a function.
    Declare(DFunc),
    // This is necessary for special Instr's such as break for control
    // flow in simple `loop ... end` blocks.
    Special(Keyword),
    Branch {
        paths: Vec<(Expr, Vec<Instr>)>,
    },
    Loop {
        body: Vec<Instr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(Lit),
    Name(String),
    Apply { left: Box<Expr>, right: Box<Expr> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lit {
    Void,
    I64(i64),
    // U64(u64),
    // F64(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Break,
    Ellipsis,
    // Instrinsic(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOP {
    Equal,
    Tilde,
}

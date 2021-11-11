use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, PartialEq, Clone)]
pub enum Prim {
    I64(i64),
    // U64(u64),
    // F64(f64),
    // Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub proc_name: String,
    pub args: Option<Vec<Expr>>,
}

#[derive(Debug, PartialEq)]
pub struct Bind {
    pub id: String,
    pub ty: String,
    pub expr: Expr,
}

// #[derive(Debug, PartialEq)]
// pub enum UnaryOp {
//     Minus,
//     Not,
// }

// #[derive(Debug, PartialEq)]
// pub enum BinaryOp {
//     Plus,
//     Minus,
//     Mult,
//     Div,
//     And,
//     Or,
// }

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Prim(Prim),
    Id(String),
    Call(Call),
    // Unary(UnaryOp, Box<Expr>),
    // Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct Proc {
    pub kind: (Option<Vec<(String, String)>>, String),
    pub body: Vec<Instr>,
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    Proc(Proc),
    // Pure(Pure),
    // Actor(Actor),
}

#[derive(Debug, PartialEq)]
pub enum Instr {
    Expr(Expr),
    Bind(Bind),
}

#[derive(Debug, PartialEq)]
pub struct AST {
    pub decls: HashMap<String, Decl>,
}

use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, PartialEq)]
pub struct AST {
    pub decls: HashMap<String, Decl>,
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    // Type(Type),
    Func(Func),
    // Actor(Actor),
}

#[derive(Debug, PartialEq)]
pub struct Func {
    // pub kind: Option<(Vec<(String, String)>, String)>,
    pub kind: Kind,
    pub body: Vec<Instr>,
}

#[derive(Debug, PartialEq)]
pub struct Kind {
    pub params: Vec<(String, String)>,
    pub ret: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    Expr(Expr),
    Bind(Bind),
    MutBind(Bind),
    Branch(Branch),
    Keyword(Keyword),
    Loop(Loop),
    Assign(Assign),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Prim(Prim),
    Name(String),
    Call(Call),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Prim {
    I64(i64),
    // U64(u64),
    // F64(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func_name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bind {
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

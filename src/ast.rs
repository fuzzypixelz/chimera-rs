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

#[derive(Debug, PartialEq)]
pub enum Instr {
    Expr(Expr),
    Bind(Bind),
    // Cond(Cond),
    // Loop(Loop),
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
    // Bool(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func_name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Bind {
    pub id: String,
    pub ty: String,
    pub expr: Expr,
}

#[derive(Debug, PartialEq)]
pub struct Cond {
    cond: Expr,
    fst: Vec<Instr>,
    snd: Vec<Instr>,
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    body: Vec<Instr>,
}

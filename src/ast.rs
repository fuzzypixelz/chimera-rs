use std::fmt::Debug;
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct AST {
    pub defs: Vec<Def>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Def {
    Type(DType),
    Name(DName),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DName {
    // A name is a reference to an expression,
    pub name: String,
    pub ann: Ann,
    pub op: AOP,
    pub expr: Expr,
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

// pub type Name   = String;
pub type Ann = Vec<String>;
// pub type Params = Vec<String>;

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    // This is meant to force evaluation of a certain Expr,
    // for example for the result at a function's end. Or,
    // for "applying" a function purely for its side-effects.
    Compute(Expr), // addOne 41
    // Create a name in the local environement that maps to a
    // mutable value. An Assign Instr allows this change.
    // NOTE: Obviously this is only allowed in impure functions.
    Bind {
        name: String,
        ann: Ann,
        op: AOP,
        expr: Expr,
    }, // var i
    Assign {
        name: String,
        op: AOP,
        expr: Expr,
    }, // i = 0
    Loop   { body: Vec<Instr> }, // loop ... end
    Branch { paths: Vec<(Expr, Vec<Instr>)> }, // if cond then 0 else 42
    // The toname: Stringplevel of a program contains function and type
    // declarexpr: Expr, ations. The same is allowed in the body of a function.
    Let(DName), // let addOne = do |x| x + 1 end
    // This is necessary for special Instr's such as break for control
    // flow in simple `loop ... end` blocks.
    Keyword(Keyword), // break
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Primitives
    Void, // ()
    I64(i64), // 42
    // U64(u64),
    // F64(f64),
    Bool(bool), // True / False
    Str(String), // "Hello, World\n"
    // Functions
    Name(String), // coolName
    Func    { param: String, body: Vec<Instr>, closure: FxHashMap<String, Expr> }, // do |x| x + 1 end
    Apply  { left: Box<Expr>, right: Box<Expr> }, // f x
    Intrinsic { name: String, args: Vec<Expr> }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Break,
    Ellipsis,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AOP {
    Equal,
    Tilde,
}


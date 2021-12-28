use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone)]
pub struct AST {
    pub module: Mod,
    pub defs: Vec<Def>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Def {
    Name(DName),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DName {
    // A name is a reference to an expression,
    pub name: String,
    pub ann: Ann,
    pub op: AssignOp,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mod {
    pub imports: Vec<String>,
    // FIXME: Allow for greater control of the
    // exported names using a `export (name1, name2)`
    // syntax.
    // exports { names: Vec<String> },
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
    Var {
        name: String,
        ann: Ann,
        op: AssignOp,
        expr: Expr,
    }, // var i
    Assign {
        name: String,
        op: AssignOp,
        expr: Expr,
    }, // i = 0
    Loop {
        body: Vec<Instr>,
    }, // loop ... end
    // The toname: Stringplevel of a program contains function and type
    // declarexpr: Expr, ations. The same is allowed in the body of a function.
    Let(DName), // let addOne = fn |x| do x + 1 end
    // This is necessary for special Instr's such as break for control
    // flow in simple `loop ... end` blocks.
    Ellipsis,
    Break,
    // Continue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Primitives
    Void,     // ()
    Int(i64), // 42
    // U64(u64),
    // F64(f64),
    Bool(bool),  // True / False
    Str(String), // "Hello, World\n"
    // Functions
    Name(String),     // coolName
    List(Vec<Expr>),  // [1, 2, 3]
    Array(Vec<Expr>), // #[1, 2, 3]
    Block {
        body: Vec<Instr>,
    },
    Func {
        param: String,
        body: Vec<Instr>,
        ann: Ann,
    }, // do |x| x + 1 end
    Apply {
        left: Box<Expr>,
        right: Box<Expr>,
    }, // f x
    Branch {
        paths: Vec<(Expr, Vec<Instr>)>,
    }, // if cond then 0 else 42
    Intrinsic {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOp {
    Equal,
    Tilde,
}

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::interpreter::Env;

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
    pub op: AOP,
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
        op: AOP,
        expr: Expr,
    }, // var i
    Assign {
        name: String,
        op: AOP,
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
    I64(i64), // 42
    // U64(u64),
    // F64(f64),
    Bool(bool),  // True / False
    Str(String), // "Hello, World\n"
    // Functions
    Name(String), // coolName
    List(List),
    Block {
        body: Vec<Instr>,
    },
    Func {
        param: String,
        body: Vec<Instr>,
        closure: Rc<RefCell<Env>>,
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
pub enum AOP {
    Equal,
    Tilde,
}

/// The representation of a "Cons List" within the interpreter,
/// as the language isn't mature enough to have custom data types yet.
/// This is a temporary way of having aggregate data types in Woland.
#[derive(Debug, PartialEq, Clone)]
pub enum List {
    Cons(Box<Expr>, Box<Expr>),
    Nil,
}

impl From<Vec<Expr>> for List {
    fn from(mut item: Vec<Expr>) -> Self {
        // FIXME: this is too slow!
        if item.is_empty() {
            List::Nil
        } else {
            // Could be better written, probably.
            let tail = item.drain(1..).collect::<Vec<Expr>>();
            let head = item.pop().unwrap();
            List::Cons(Box::new(head), Box::new(Expr::List(tail.into())))
        }
    }
}

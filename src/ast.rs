use polytype::TypeSchema;
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
    // FIXME: Name definition, restricted to the block scope it appears in,
    // it can be a toplevel item or an Instr in a block. This is a bit
    // awkward and it might make more sense to make the AST a list of Instr's
    // intead.
    pub name: String,
    pub ann: Option<TypeSchema>,
    // The assignement operator specifies whether evaluating the expr
    // to the right produces side-effects or not, `=` is for pure expr's
    // while `~` is for impure ones. The idea is that this would be statically
    // checked by the compiler for inconsistencies.
    // TODO: Figure out a way to expand the syntax to specify different kinds
    // of effects. Effect systems are already present in languages such as Koka
    // and Unison. Ideally, we would also want a way to compose multiple effects
    // into one and handle them accordinly upon expression evaluation.
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

#[derive(Debug, PartialEq, Clone)]
pub enum Instr {
    // This is meant to force evaluation of a certain Expr,
    // for example for the result at a function's end. Or,
    // for "applying" a function purely for its side-effects.
    Compute(Expr), // addOne 41
    // Create a name in the local environement that maps to a
    // mutable value. An Assign Instr allows this change.
    // NOTE: Obviously this is only allowed in impure functions.
    // TODO: Once we have mutable data types we can define things like:
    //           type Cell a = Cell { mut value: ref a }
    // Then, it makes sense to compile a `let mut x = 3` down to something like:
    //           let x = Cell { value: ref 3 }
    // This way we avoid dealing with the shenanigans of variables; each name points
    // to an immutable instance of a data-type, and its the _type_ that defines its
    // own mutability. These are the _roughly_ the semantics of F#/OCaml.
    Var {
        name: String,
        ann: Option<TypeSchema>,
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
    Let(DName), // let addOne = |x| x + 1
    Ellipsis,   // ...
    Break,      // break
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Primitives
    Void,     // ()
    Int(i64), // 42
    // U64(u64),
    // F64(f64),
    Bool(bool),  // true / false
    Str(String), // "Hello, World"
    Char(char),  // 'c'
    // Functions
    Name(String),     // coolName
    List(Vec<Expr>),  // [1, 2, 3]
    Array(Vec<Expr>), // #[1, 2, 3]
    // TODO: enforce the fact that the last Instr in a code-block
    // should be an Expr by baking it into the parser, producing
    // a seperate `expr` field here. This will simplify the
    // type-checker somewhat.
    Block { body: Vec<Instr> },
    Func { param: String, expr: Box<Expr> },     // |x| x + 1
    Apply { left: Box<Expr>, right: Box<Expr> }, // f x
    Branch { paths: Vec<(Expr, Vec<Instr>)> },   // if cond then 0 else 42 end
    Intrinsic { name: String, args: Vec<Expr> }, // @intrinsic arg0 ... argN
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOp {
    Equal,
    Tilde,
}

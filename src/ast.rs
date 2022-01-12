use polytype::TypeSchema;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Definition {
        name: String,
        ann: Option<TypeSchema>,
        expr: Expr,
    },
    Attribute {
        name: String,
        args: Vec<Expr>,
    },
    DataType {
        kind: TypeSchema,
        variants: Vec<(String, Vec<(String, TypeSchema)>)>,
    },
    Module {
        name: String,
        items: Vec<Item>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Item(Item),
    // This is meant to force evaluation of a certain Expr,
    // for example for the result at a function's end. Or,
    // for "applying" a function purely for its side-effects.
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Primitives
    Void,     // ()
    Int(i64), // 42
    // U64(u64),
    // F64(f64),
    Bool(bool),  // true / false
    Char(char),  // 'c'
    Str(String), // "Hello, World"
    // Functions
    Name(String), // coolName
    // TODO: enforce the fact that the last Instr in a code-block
    // should be an Expr by baking it into the parser, producing
    // a seperate `expr` field here. This will simplify the
    // type-checker somewhat.
    Lambda { param: String, expr: Box<Expr> }, // |x| x + 1
    Block { body: Vec<Stmt> },
    Apply { left: Box<Expr>, right: Box<Expr> }, // f x
    Branch { paths: Vec<(Expr, Vec<Stmt>)> },    // if cond then 0 else 42 end
    Field { expr: Box<Expr>, name: String },
    Assign { left: Box<Expr>, right: Box<Expr> }, // i = 0
}

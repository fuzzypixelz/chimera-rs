/// Chimera's abstract syntax tree.
/// The layout is highly inspired by rustc's own ast.
use polytype::TypeSchema;

#[derive(Debug, PartialEq, Clone)]
pub struct AST {
    pub items: Vec<Item>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Attr {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item {
    pub attr: Option<Attr>,
    pub kind: ItemKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ItemKind {
    Definition {
        name: String,
        ann: Option<TypeSchema>,
        expr: Expr,
    },
    DataType {
        schema: TypeSchema,
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
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    // Primitives
    Ellipsis,
    Void,
    // ()
    Int(i64),
    // 42
    // U64(u64),
    // F64(f64),
    Bool(bool),
    // true / false
    Char(char),
    // 'c'
    // Functions
    Name(String),
    List(Vec<Expr>),
    // coolName
    // TODO: enforce the fact that the last Instr in a code-block
    // should be an Expr by baking it into the parser, producing
    // a seperate `expr` field here. This will simplify the
    // type-checker somewhat.
    Lambda { param: String, expr: Box<Expr> },
    // |x| x + 1
    Block { body: Vec<Stmt> },
    Apply { left: Box<Expr>, right: Box<Expr> },
    // f x
    Branch { paths: Vec<(Expr, Vec<Stmt>)> },
    // if cond then 0 else 42 end
    Field { expr: Box<Expr>, name: String },
    Assign { left: Box<Expr>, right: Box<Expr> }, // i = 0
}

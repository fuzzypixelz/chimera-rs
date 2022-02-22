use polytype::TypeSchema;

/// Abstract Syntax Tree, mirrors the syntax closely with no transformations.
#[derive(Debug)]
pub struct AST {
    pub items: Vec<Node<Item>>,
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub span: Span,
    pub attr: Option<Attr>,
    pub inner: T,
}

#[derive(Debug, Clone)]
pub enum Item {
    Definition {
        name: String,
        ann: Option<TypeSchema>,
        expr: Node<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Void,
    Lit(Lit),
    Name(String),
    Lambda {
        params: Vec<String>,
        expr: Box<Node<Expr>>,
    },
    Apply {
        left: Box<Node<Expr>>,
        right: Vec<Node<Expr>>,
    },
    Block {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum Lit {
    Int(i64),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Item(Node<Item>),
    Expr(Node<Expr>),
}

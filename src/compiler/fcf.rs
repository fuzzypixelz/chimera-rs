use polytype::Type;

use crate::{
    check::cst::{self, TypedExpr, TypedParam},
    parser::ast::Lit,
};

use super::ccf::{Definition, CCF};

/// Flat Chimera Form, lambdas are lifted into top level functions.
#[derive(Debug)]
pub struct FCF {
    /// Top-level bindings with eliminated closures.
    pub binds: Vec<Bind>,
    /// Extracted functions via lambda-lifting.
    pub funcs: Vec<Func>,
    // Main function.
    pub main: Func,
}

#[derive(Debug)]
pub struct Bind {
    /// User-given name for the expression.
    pub name: String,
    /// A non-lambda expression.
    pub fexpr: FlatExpr,
}

#[derive(Debug)]
pub struct Func {
    /// Function signature name.
    pub name: String,
    /// Functions parameter name.
    pub tparams: Vec<TypedParam>,
    /// Expression returned by the function.
    pub fexpr: FlatExpr,
}

#[derive(Debug)]
pub struct FlatExpr {
    kind: Type,
    expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Void,
    Lit(Lit),
    Name(String),
    Apply {
        fleft: Box<FlatExpr>,
        fright: Vec<FlatExpr>,
    },
    Call {
        fleft: Box<FlatExpr>,
        fright: Vec<FlatExpr>,
    },
    Block {
        body: Vec<Bind>,
    },
}

impl TypedExpr {
    fn flatten(self, name: &str, funcs: &mut Vec<Func>, lifted: usize) -> FlatExpr {
        let TypedExpr { kind, expr } = self;
        match expr {
            cst::Expr::Void => FlatExpr {
                kind,
                expr: Expr::Void,
            },
            cst::Expr::Lit(lit) => FlatExpr {
                kind,
                expr: Expr::Lit(lit.clone()),
            },
            cst::Expr::Name(name) => FlatExpr {
                kind,
                expr: Expr::Name(name.clone()),
            },
            cst::Expr::Lambda { texpr, tparams } => {
                let name = format!("{name}_$_{lifted}");
                let func = Func {
                    fexpr: texpr.flatten(&name, funcs, lifted + 1),
                    name: name.clone(),
                    tparams: tparams.clone(),
                };
                funcs.push(func);
                FlatExpr {
                    kind,
                    expr: Expr::Name(name.clone()),
                }
            }
            cst::Expr::Call { tleft, tright } => {
                let fleft = Box::new(tleft.flatten(name, funcs, lifted));
                let fright = tright
                    .into_iter()
                    .map(|texpr| texpr.flatten(name, funcs, lifted))
                    .collect::<Vec<_>>();
                FlatExpr {
                    kind,
                    expr: Expr::Call { fleft, fright },
                }
            }
            cst::Expr::Apply { tleft, tright } => {
                let fleft = Box::new(tleft.flatten(name, funcs, lifted));
                let fright = tright
                    .into_iter()
                    .map(|texpr| texpr.flatten(name, funcs, lifted))
                    .collect::<Vec<_>>();
                FlatExpr {
                    kind,
                    expr: Expr::Apply { fleft, fright },
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl From<CCF> for FCF {
    fn from(ccf: CCF) -> Self {
        let mut binds = Vec::new();
        let mut funcs = Vec::new();

        for Definition { name, texpr } in ccf.defs {
            let fexpr = texpr.flatten(&name, &mut funcs, 0);
            binds.push(Bind { name, fexpr });
        }
        let main = Func {
            name: "main".to_string(),
            tparams: vec![],
            fexpr: ccf.main.flatten("main", &mut funcs, 0),
        };

        FCF { binds, funcs, main }
    }
}

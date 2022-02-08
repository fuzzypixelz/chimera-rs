use crate::parser::cst::{Def, Expr, ExprKind, Lit};

use super::ccf::CCF;

/// Flat Chimera Form, lambdas are lifted into top level functions.
#[derive(Debug, Default)]
pub struct FCF {
    /// Top-level bindings with eliminated closures.
    pub binds: Vec<Bind>,
    /// Extracted functions via lambda-lifting.
    pub funcs: Vec<Func>,
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
    pub param: String,
    /// Expression returned by the function.
    pub fexpr: FlatExpr,
}

/// A non-lambda expression.
#[derive(Debug)]
pub enum FlatExpr {
    /// Literal expression.
    Lit(Lit),
    /// Variable expression, referes to a name.
    Var(String),
    Call(String, Box<FlatExpr>),
}

impl FCF {
    /// Consumes a definition by transforming it into a `Bind` or a `Func`.
    fn consume(&mut self, def: Def) {
        let Def { name, expr } = def;
        match expr.kind {
            ExprKind::Lit(lit) => self.binds.push(Bind {
                name,
                fexpr: FlatExpr::Lit(lit),
            }),
            ExprKind::Var(var) => self.binds.push(Bind {
                name,
                fexpr: FlatExpr::Var(var),
            }),
            ExprKind::App(fname, lit) => self.binds.push(Bind {
                name,
                fexpr: FlatExpr::Call(fname, Box::new(FlatExpr::Lit(lit))),
            }),
            ExprKind::Lam(param, expr) => {
                let fexpr = self.flatten(&name, *expr, 0);
                self.funcs.push(Func { name, param, fexpr });
            }
        }
    }

    /// Lifts nested lambdas by replacing them with `Var` expressions
    /// that reference a top-level function.
    fn flatten(&mut self, func_name: &str, expr: Expr, lifted: usize) -> FlatExpr {
        match expr.kind {
            ExprKind::Lit(lit) => FlatExpr::Lit(lit),
            ExprKind::Var(var) => FlatExpr::Var(var),
            ExprKind::App(fname, lit) => FlatExpr::Call(fname, Box::new(FlatExpr::Lit(lit))),
            ExprKind::Lam(param, expr) => {
                // FIXME: There is a possibility of collision if we
                // start userspace definition names with "__".
                // This should be enforced in the lexer as it makes for
                // bad style anyway (at least for now).
                let name = format!("__{func_name}_{lifted}");
                let fexpr = self.flatten(&name, *expr, lifted + 1);
                self.funcs.push(Func {
                    name: name.clone(),
                    param,
                    fexpr,
                });
                FlatExpr::Var(name)
            }
        }
    }
}

impl From<CCF> for FCF {
    fn from(ccf: CCF) -> Self {
        let mut fcf = FCF::default();
        for def in ccf.defs {
            fcf.consume(def);
        }
        // let main: () -> () = |_| (* actual "main" expression *)
        let fexpr = fcf.flatten("main", ccf.main, 0);
        fcf.funcs.push(Func {
            name: "main".to_string(),
            param: "_".to_string(),
            fexpr,
        });
        fcf
    }
}

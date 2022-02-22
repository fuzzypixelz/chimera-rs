pub mod cst;

use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::Result;
use polytype::{tp, Context, Type, TypeSchema};

use self::cst::{TypedExpr, TypedParam, CST};
use crate::error::TypeError;
use crate::parser::ast::{Expr, Item, Lit, Node, Stmt, AST};

#[derive(Default)]
pub struct Lexicon<'a> {
    // NOTE: `Context` is a bit of a misnomer: it doesn't track information
    // about the types of (variable) names, but rather the substitutions done
    // when you call `.unify()` and friends.
    ctx: RefCell<Context>,
    assumptions: RefCell<HashMap<String, TypeSchema>>,
    // TODO: Is there a use for keeping track of infered types?
    // Maybe we could put them back into the AST and provide them
    // to the user on demand. As it stands now, we are only making
    // use of the type-checking ability of algorith J.
    // types: RefCell<HashMap<String, Type>>,
    outer: Option<&'a Lexicon<'a>>,
}

impl Lexicon<'_> {
    /// The Type inference algorithm is called J, for some reason.
    ///
    /// According to Milner:
    ///   > As it stands, W is hardly an efficient algorithm;
    ///   substitutions are applied too often.
    ///
    /// There is a case for each possible Expr in the AST, the main
    /// ones that make up a lambda calculus are Name, Apply, Func and Let;
    /// the program can be thought of as a sequence of the last case:
    ///
    /// `let name0 = expr0 in let name1 = expr1 in ... let nameN = exprN in ()`.
    ///
    /// This means that Chimera's Let-syntax is different from the polymorphic
    /// lambda calculus' Let-polymorphism, but is still equivalent to it.
    fn infer(&self, expr: Node<Expr>) -> Result<TypedExpr, TypeError> {
        match expr.inner {
            // Boring hard-coded primitive types, nothing to see here!
            Expr::Void => Ok(TypedExpr {
                kind: tp!(Void),
                expr: cst::Expr::Void,
            }),
            Expr::Lit(lit) => match lit {
                Lit::Int(_) => Ok(TypedExpr {
                    kind: tp!(Int),
                    expr: cst::Expr::Lit(lit),
                }),
            },
            // This corresponds to the [VAR] rule:
            // We check the lexicon for an assumption about `name` which gives us
            // a polytype `ts`, otherwise the algorithm fails.
            // We then specialize `ts` to a monotype `t` by replacing the bounded type
            // variables by fresh new ones; `t` is then the type of `name`.
            Expr::Name(name) => Ok(TypedExpr {
                kind: self.get(&name)?,
                expr: cst::Expr::Name(name),
            }),
            // This corresponds to the [APP] rule:
            // Only this rule forces refinement of the type variables introduced.
            // We recursively call J to infer the type of `left` and `right`,
            // we call them `tl` and `tr` respectively (very inventive).
            // Then we make a fresh type variable for the application result `ta`,
            // and we try to unify `tl` with `tr -> ta`. That is we check that
            // left is a function that takes a right and produces _something_,
            // if successful, we determine the type of the resuling expression: `ta`,
            // by applying to it the subsititutions that follow from the Unification.
            Expr::Apply { left, right } => {
                let tleft = Box::new(self.infer(*left)?);
                let mut tright = Vec::new();
                for arg in right {
                    tright.push(self.infer(arg)?);
                }
                let mut ctx = self.ctx.borrow_mut();
                let result = ctx.new_variable();
                let right_arrow = tright.iter().map(|e| e.kind.clone()).collect::<Vec<_>>();
                ctx.unify(
                    &tleft.kind,
                    &Type::arrow(right_arrow.into(), result.clone()),
                )?;
                // We know at this point that left is really a Lambda.
                let kind = result.apply(&ctx);
                let left_arrow = tleft.kind.args().unwrap();
                let expr = if left_arrow.len() == tright.len() {
                    cst::Expr::Call { tleft, tright }
                } else {
                    cst::Expr::Apply { tleft, tright }
                };
                Ok(TypedExpr { kind, expr })
            }
            // This corresponds to the [ABS] rule (it stands for abstraction):
            // We start by generating a fresh type variable `tp` and assign it to the
            // function parameter, which is added as an assumption in the lexicon.
            // Then we use the new information to infer the type of `expr`, say `te`.
            // If successful, we know that the lambda is of type `tp -> te`.
            Expr::Lambda { params, expr } => {
                let mut tparams = Vec::new();
                for param in &params {
                    let kind = self.ctx.borrow_mut().new_variable();
                    self.assumptions
                        .borrow_mut()
                        .insert(param.clone(), TypeSchema::Monotype(kind.clone()));
                    tparams.push(TypedParam {
                        kind,
                        param: param.clone(),
                    });
                }
                let texpr = Box::new(self.infer(*expr)?);
                // The assumption about the parameter is no longer of any use,
                // and should be removed before continuing. Otherwise it would
                // pollute the namespace and lead to contradictions.
                // NOTE: this is not explicit in the description of algorithm J.
                for param in params {
                    self.assumptions.borrow_mut().remove_entry(&param);
                }
                let param_arrow = tparams.iter().map(|e| e.kind.clone()).collect::<Vec<_>>();
                let kind = Type::arrow(param_arrow.into(), texpr.kind.clone());
                let expr = cst::Expr::Lambda { tparams, texpr };
                Ok(TypedExpr { kind, expr })
            }
            // If the last block is a statement-expression, then that determines
            // the type of the block, otherwise a Void type is assumed.
            // Currently all Item's are ignored.
            Expr::Block { mut body } => {
                let local_lexicon = Lexicon {
                    outer: Some(self),
                    ..Default::default()
                };
                let last = body.pop().unwrap();
                for stmt in body {
                    match stmt {
                        Stmt::Expr(expr) => {
                            local_lexicon.infer(expr)?;
                        }
                        Stmt::Item(item) => {
                            // TODO: make a Check Trait for items.
                            local_lexicon.check(item)?;
                        }
                    }
                }
                if let Stmt::Expr(expr) = last {
                    local_lexicon.infer(expr)
                } else {
                    Ok(TypedExpr {
                        kind: tp!(Void),
                        expr: cst::Expr::Void,
                    })
                }
            }
            _ => unimplemented!("the expression {:?} is not type-checked!", expr),
        }
    }

    pub fn get(&self, name: &str) -> Result<Type, TypeError> {
        match self.assumptions.borrow().get(name) {
            None => match self.outer {
                None => Err(TypeError::ScopeError(name.to_string())),
                Some(lexicon) => lexicon.get(name),
            },
            Some(ts) => Ok(ts.instantiate(&mut self.ctx.borrow_mut())),
        }
    }

    pub fn check(&self, item: Node<Item>) -> Result<cst::Item, TypeError> {
        match item.inner {
            Item::Definition { name, ann, expr } => {
                // This corresponds to the [LET] rule:
                // We first find the most general type `te` for `expr`,
                // Then we "clone" the type `te` by universally quantifying
                // all the free type variables within it that are NOT also
                // free in the assumptions, the resulting polytype `ts` is
                // then added to the assumptions as the type of `name`.
                let texpr = self.infer(expr)?;
                let free_variables = self
                    .assumptions
                    .borrow()
                    .values()
                    .flat_map(|ts| ts.free_vars())
                    .collect::<Vec<_>>();
                // Variables specified by `bound` remain unquantified.
                let ts = texpr.kind.generalize(&free_variables);
                if let Some(ann_ts) = ann {
                    if !ann_ts.eq(&ts) {
                        panic!("incompatible annotation")
                    }
                }
                self.assumptions.borrow_mut().insert(name.clone(), ts);
                Ok(cst::Item::Definition {
                    name: name.clone(),
                    texpr,
                })
            }
            _ => unimplemented!("the item {:?} is not type-checked!", item),
        }
    }
}

impl TryFrom<AST> for CST {
    type Error = TypeError;
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        let lexicon = Lexicon::default();
        let mut items = Vec::with_capacity(ast.items.len());
        for item in ast.items {
            items.push(lexicon.check(item)?);
        }
        Ok(CST { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Span;

    /*
    #[test]
    fn block_explicit_return() {
        let lexicon = Lexicon::default();
        let block = Node {
            kind: Expr::Block {
                body: vec![
                    Stmt::Item(Node {
                        kind: Item::Definition {
                            name: "answer".to_string(),
                            ann: None,
                            expr: Node {
                                kind: Expr::Lit(Lit::Int(42)),
                                span: Span { start: 0, end: 0 },
                                attr: None,
                            },
                        },
                        span: Span { start: 0, end: 0 },
                        attr: None,
                    }),
                    Stmt::Expr(Node {
                        kind: Expr::Name("answer".to_string()),
                        span: Span { start: 0, end: 0 },
                        attr: None,
                    }),
                ],
            },
            span: Span { start: 0, end: 0 },
            attr: None,
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Int)));
    }

    #[test]
    fn block_implicit_return() {
        let lexicon = Lexicon::default();
        let block = Expr {
            kind: ExprKind::Block {
                body: vec![Stmt::Item(Item {
                    kind: ItemKind::Definition {
                        name: "zero".to_string(),
                        ann: None,
                        expr: Expr {
                            kind: ExprKind::Lit(Lit {
                                kind: LitKind::Int(0),
                            }),
                        },
                    },
                })],
            },
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Void)));
    }

    #[test]
    fn block_nested_block_expr() {
        let lexicon = Lexicon::default();
        let block = Expr {
            kind: ExprKind::Block {
                body: vec![
                    Stmt::Item(Item {
                        kind: ItemKind::Definition {
                            name: "shadowed".to_string(),
                            ann: None,
                            expr: Expr {
                                kind: ExprKind::Lit(Lit {
                                    kind: LitKind::Int(1),
                                }),
                            },
                        },
                    }),
                    Stmt::Expr(Expr {
                        kind: ExprKind::Block {
                            body: vec![
                                Stmt::Item(Item {
                                    kind: ItemKind::Definition {
                                        name: "shadowed".to_string(),
                                        ann: None,
                                        expr: Expr {
                                            kind: ExprKind::Void,
                                        },
                                    },
                                }),
                                Stmt::Expr(Expr {
                                    kind: ExprKind::Name("shadowed".to_string()),
                                }),
                            ],
                        },
                    }),
                ],
            },
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Void)));
    }
    */
}

use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;
use polytype::{Context, Infer, tp, Type, TypeSchema};

use crate::ast::{Expr, Item, ItemKind, Stmt};
use crate::error::TypeError;

#[derive(Default, Clone)]
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

// See: https://en.wikipedia.org/wiki/Hindley-Milner_type_system#Algorithm_J
impl<'a> Infer<Lexicon<'a>, TypeError> for Expr {
    // The Type inference algorithm is called J, for some reason.
    // NOTE: According to Milner:
    //   "As it stands, W is hardly an efficient algorithm;
    //   substitutions are applied too often."
    // There is a case for each possible Expr in the AST, the main
    // ones that make up a lambda calculus are Name, Apply, Func and Let;
    // the program can be thought of as a sequence of the last case:
    // `let name0 = expr0 in let name1 = expr1 in ... let nameN = exprN in ()`.
    // This means that Chimera's Let-syntax is different from the polymorphic
    // lambda calculus' Let-polymorphism, but is still equivalent to it.
    fn infer(&self, lexicon: &Lexicon<'a>) -> Result<Type, TypeError> {
        match &self {
            // Boring hard-coded primitive types, nothing to see here!
            Expr::Void => Ok(tp!(Void)),
            Expr::Int(_) => Ok(tp!(Int)),
            Expr::Bool(_) => Ok(tp!(Bool)),
            Expr::Char(_) => Ok(tp!(Char)),
            // This corresponds to the [VAR] rule:
            // We check the lexicon for an assumption about `name` which gives us
            // a polytype `ts`, otherwise the algorithm fails.
            // We then specialize `ts` to a monotype `t` by replacing the bounded type
            // variables by fresh new ones; `t` is then the type of `name`.
            Expr::Name(name) => lexicon.get(&name),
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
                let tl = left.infer(lexicon)?;
                let tr = right.infer(lexicon)?;
                let mut ctx = lexicon.ctx.borrow_mut();
                let ta = ctx.new_variable();
                ctx.unify(&tl, &Type::arrow(tr, ta.clone()))?;
                Ok(ta.apply(&ctx))
            }
            // This corresponds to the [ABS] rule (it stands for abstraction):
            // We start by generating a fresh type variable `tp` and assign it to the
            // function parameter, which is added as an assumption in the lexicon.
            // Then we use the new information to infer the type of `expr`, say `te`.
            // If successful, we know that the lambda is of type `tp -> te`.
            Expr::Lambda { param, expr } => {
                let tp = lexicon.ctx.borrow_mut().new_variable();
                lexicon
                    .assumptions
                    .borrow_mut()
                    .insert(param.clone(), TypeSchema::Monotype(tp.clone()));
                let te = expr.infer(lexicon)?;
                // The assumption about the parameter is no longer of any use,
                // and should be removed before continuing. Otherwise it would
                // pollute the namespace and lead to contradictions.
                // NOTE: this is not explicit in the description of algorithm J.
                lexicon.assumptions.borrow_mut().remove_entry(param);
                Ok(Type::arrow(tp, te))
            }
            // If the last block is a statement-expression, then that determines
            // the type of the block, otherwise a Void type is assumed.
            // Currently all Item's are ignored.
            Expr::Block { body } => {
                let mut local_lexicon = Lexicon::default();
                // FIXME: this clones the entire "list" of lexicons
                // each time it entered a new block, not good.
                local_lexicon.outer = Some(lexicon);
                let (last, init) = body.split_last().unwrap();
                for stmt in init {
                    match stmt {
                        Stmt::Expr(expr) => {
                            expr.infer(&local_lexicon)?;
                        }
                        Stmt::Item(item) => {
                            // TODO: make a Check Trait for items.
                            local_lexicon.check(item)?;
                        }
                    }
                }
                if let Stmt::Expr(expr) = last {
                    expr.infer(&local_lexicon)
                } else {
                    Ok(tp!(Void))
                }
            }
            _ => unimplemented!("the expression {:?} is not type-checked!", self),
        }
    }
}

impl<'a> Lexicon<'a> {
    pub fn get(&self, name: &str) -> Result<Type, TypeError> {
        match self.assumptions.borrow().get(name) {
            None => match self.outer {
                None => Err(TypeError::ScopeError(name.to_string())),
                Some(l) => l.get(name),
            },
            Some(ts) => Ok(ts.instantiate(&mut self.ctx.borrow_mut())),
        }
    }

    pub fn check(&self, item: &Item) -> Result<(), TypeError> {
        match &item.kind {
            ItemKind::Module { items, .. } => {
                for item in items {
                    self.check(item)?;
                }
            }
            ItemKind::Definition { name, ann, expr } => {
                if let Some(ts) = ann {
                    // We introduce type annotations into the algorithm
                    // without any checks, as if it it infered them itself.
                    // It is up to the user to make sure their annotations
                    // are correct, that is if they choose to add them.
                    // Annotations are almost always unecessary. Keep It Simple.
                    self.assumptions
                        .borrow_mut()
                        .insert(name.clone(), ts.clone());
                } else {
                    // This corresponds to the [LET] rule:
                    // We first find the most general type `te` for `expr`,
                    // Then we "clone" the type `te` by universally quantifying
                    // all the free type variables within it that are NOT also
                    // free in the assumptions, the resulting polytype `ts` is
                    // then added to the assumptions as the type of `name`.
                    let te = expr.infer(&self)?;
                    let free_variables = self
                        .assumptions
                        .borrow()
                        .values()
                        .flat_map(|ts| ts.free_vars())
                        .collect::<Vec<_>>();
                    // Variables specified by `bound` remain unquantified.
                    let ts = te.generalize(&free_variables);
                    self.assumptions.borrow_mut().insert(name.clone(), ts);
                }
            }
            _ => unimplemented!("the item {:?} is not type-checked!", item),
        }
        for (name, ts) in self.assumptions.borrow().iter() {
            println!("[@typechecker] {} : {}", name, ts)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_explicit_return() {
        let lexicon = Lexicon::default();
        let block = Expr::Block {
            body: vec![
                Stmt::Item(Item {
                    kind: ItemKind::Definition {
                        name: "answer".to_string(),
                        ann: None,
                        expr: Expr::Int(42),
                    },
                    attrs: vec![],
                }),
                Stmt::Expr(Expr::Name("answer".to_string())),
            ],
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Int)));
    }

    #[test]
    fn block_implicit_return() {
        let lexicon = Lexicon::default();
        let block = Expr::Block {
            body: vec![Stmt::Item(Item {
                kind: ItemKind::Definition {
                    name: "chimera".to_string(),
                    ann: None,
                    expr: Expr::Str("monstrous fire-breathing hybrid creature".to_string()),
                },
                attrs: vec![],
            })],
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Void)));
    }

    #[test]
    fn block_nested_block_expr() {
        let lexicon = Lexicon::default();
        let block = Expr::Block {
            body: vec![
                Stmt::Item(Item {
                    kind: ItemKind::Definition {
                        name: "shadowed".to_string(),
                        ann: None,
                        expr: Expr::Bool(true),
                    },
                    attrs: vec![],
                }),
                Stmt::Expr(Expr::Block {
                    body: vec![
                        Stmt::Item(Item {
                            kind: ItemKind::Definition {
                                name: "shadowed".to_string(),
                                ann: None,
                                expr: Expr::Int(0),
                            },
                            attrs: vec![],
                        }),
                        Stmt::Expr(Expr::Name("shadowed".to_string())),
                    ],
                }),
            ],
        };
        assert_eq!(block.infer(&lexicon), Ok(tp!(Int)));
    }
}

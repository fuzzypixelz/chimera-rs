//!   This file is the Woland Tree-Walk Interpreter.
//!
//!   Following the parser's generation of the AST, this
//!   module executes the program described by the tree
//!   directly, i.e no IR is used.
//!
//!   Hence, it contains the following facilities for
//!   simulating a runtime:
//!
//!   # Environments.
//!
//!     1. A notion of proc/pure function local environment,
//!     consisting of a HashMap between binding names and Expr's.
//!     2. A global HashMap mapping each proc/pure function's
//!     name to a local environment.
//!
//!   # Expression evaluation.
//!
//!     1. A means of evaluating any Expr (recursively), this would
//!     require access to the proc/pure's local environment.
//!     2. Currently all evaluation produces only Woland's primitive
//!     types. In the future there will be support for compound data
//!     types as well as abstract data types. The current primitives
//!     are: Bool, I64, U64, F64, String.
//!
//!   # Proc execution.
//!     1. As it stands, a proc is a sequence of Instr's:
//!     enum Instr {
//!         Expr(Expr), // Produces a value (B) and may have side-
//!                     // effects, useful for returns and calls.
//!         Bind(Bind), // Changes the proc's local env, useful
//!                     // for giving names to Expr's.
//!     }
//!     2. A procedure's last instruction will always be an Expr,
//!     this is what we evaluate its Calls to.
//!
//!   # Imports
//!     1. importing a file means parsing it into an AST, and including
//!     only the `decls` presents in its exports section, otheriwse
//!     we include everything by "merging" the two `decls` fields.

use rustc_hash::FxHashMap;
use std::fmt::Display;
use std::io::{self, Read};

use crate::ast::*;

#[derive(Clone)]
pub struct Env {
    names: FxHashMap<String, Expr>,
    // types: FxHashMap<String, Type>,
    vars: FxHashMap<String, Expr>,
    loops: u64,
}

impl Env {
    pub fn new(ast: &AST) -> Self {
        let mut env = Self {
            names: FxHashMap::default(),
            vars: FxHashMap::default(),
            loops: 0,
        };

        for def in ast.to_owned().defs {
            match def {
                Def::Name(dname) => {
                    env.names.insert(dname.name, dname.expr);
                }
                _ => unimplemented!()
            }
        }
        env
    }

    pub fn get(&self, item: &str) -> &Expr {
        match self.names.get(item) {
            None => match self.vars.get(item) {
                None => panic!("Woland: undefined reference to {} name.", item),
                Some(e) => e,
            },
            Some(e) => e,
        }
    }
}

impl Expr {
    pub fn eval<'a>(&'a self, env: &'a Env) -> Expr {
        match self {
            Expr::Void     => Expr::Void,
            Expr::I64(i)   => Expr::I64(*i),
            Expr::Bool(b)  => Expr::Bool(*b),
            Expr::Str(s)   => Expr::Str(s.to_owned()),
            Expr::Name(id) => env.get(id).eval(env),
            Expr::Func { param, body, closure: _ } => Expr::Func { 
                param: param.to_owned(), 
                body: body.to_owned(),
                closure: env.names.to_owned()
            },
            Expr::Apply { left, right } => {
                if let Expr::Func { param, body, closure } = left.eval(env) {
                    let mut fenv = env.to_owned();
                    fenv.names.insert(param, right.eval(env));
                    // TODO: make sure the closure names have priority
                    fenv.names.extend(closure);

                    // NOTE: the typechecker should've already ensured
                    // the body is not empty, so unwrap away!
                    let (last, init) = body.split_last().unwrap();
                    for instr in init {
                        instr.execute(&mut fenv);
                    }
                    last.execute(&mut fenv).eval(&fenv)
                } else {
                    // TODO: switch all unreachable!'s to instrinsics
                    unreachable!();
                }
            }
            Expr::Intrinsic { name, args } => match name.as_str() {
                "dump" => {
                    println!("{}", args[0].eval(env));
                    Expr::Void
                }
                "read" => {
                    let mut buffer = String::new();
                    io::stdin()
                        .read_to_string(&mut buffer)
                        // .read_line(&mut buffer)
                        .expect("Woland: error reading from stdin. You are on your own.");
                    Expr::Str(buffer)
                }
                "cmp" => {
                    Expr::Bool(args[0].eval(env) == args[1].eval(env))
                }
                "add" => {
                    if let Expr::I64(l) = args[0].eval(env) {
                        if let Expr::I64(r) = args[1].eval(env) {
                                Expr::I64(l + r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "mod" => {
                    if let Expr::I64(l) = args[0].eval(env) {
                        if let Expr::I64(r) = args[1].eval(env) {
                                Expr::I64(l % r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => Expr::Void
            }
        }
    }
}

impl Instr {
    fn execute(&self, env: &mut Env) -> Expr {
        match self {
            Instr::Compute(expr) => {
                // The evaluated expression may or may not have
                // any side-effects. Beware!
                expr.eval(env)
            }
            Instr::Var { name, ann: _, op: _, expr  } => {
                env.vars.insert(name.to_owned(), expr.eval(env));
                Expr::Void
            }
            Instr::Assign { name, op: _, expr } => {
                if !env.vars.contains_key(name) {
                    panic!("Woland: {} is not a mutable name.", name)
                }
                // Imperative assignment enforces strict evalation,
                // otherwise we can't do simple Assign's such as
                // i = @addI64 i 1
                env.vars.insert(name.to_owned(), expr.eval(env));
                Expr::Void
            }
            Instr::Let(dname) => {
                env.names.insert(
                    dname.name.to_owned(),
                    dname.expr.to_owned()
                );
                dname.expr.to_owned()
            }

            Instr::Loop { body } => {
                env.loops += 1;
                let start = env.loops;
                while start == env.loops {
                    // body.iter().for_each(|i| { i.execute(env); })
                    for i in body {
                        i.execute(env);
                        if env.loops != start {
                            break
                        }
                    }
                };
                Expr::Void
            }
            
            Instr::Branch { paths } => {
                let mut result = Expr::Void;
                for p in paths {
                    if let Expr::Bool(b) = p.0.eval(env) {
                        if b {
                            let (last, init) = p.1.split_last().unwrap();
                            for i in init {
                                i.execute(env);
                            }
                            result = last.execute(env);
                            break
                        }
                    } else {
                        unreachable!()
                    }
                }
                result
                // TODO: Make branches evaluate to the last expression Compute'd
            }
            Instr::Keyword(keyword) => match keyword {
                Keyword::Break => {
                    if env.loops == 0 {
                        panic!("Woland: can only break out of a loop.")
                    }
                    env.loops -= 1;
                    Expr::Void
                }
                Keyword::Ellipsis => { 
                    /* Do nothing! This is a simple filler Instr */ 
                    Expr::Void
                }
            },
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::I64(i)  => write!(f, "{}", i),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Str(s)  => write!(f, "{}", s),
            other => write!(f, "{:#?}", other),
        }
    }
}

#[cfg(test)]
mod tests {
}

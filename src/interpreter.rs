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
use std::cell::RefCell;
use std::fmt::Display;
use std::io::{self, Read};
use std::rc::Rc;

use crate::ast::*;

#[derive(Clone)]
pub struct Cont {
    // A "Continuation" i.e
    // the evaluator's state.
    loops: u64,
}

impl Cont {
    pub fn default() -> Self {
        Self { loops: 0 }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Env {
    // types: FxHashMap<String, Type>,
    // FIXME: this should not be public!
    pub names: FxHashMap<String, Expr>,
    vars: FxHashMap<String, Expr>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new(defs: &Vec<Def>) -> Self {
        let mut env = Self::default();
        for def in defs {
            match def {
                Def::Name(dname) => {
                    env.names.insert(dname.name.clone(), dname.expr.clone());
                }
            }
        }
        env
    }

    /// Get the first Env containing `name`, this was we can mutate the var directly.
    pub fn get_var_env(env: Rc<RefCell<Env>>, name: &str) -> Rc<RefCell<Env>> {
        if !env.borrow().vars.contains_key(name) {
            match env.borrow().outer.clone() {
                None => panic!("Woland: `{}` is not a defined mutable name.", name),
                Some(oenv) => Self::get_var_env(oenv, name),
            }
        } else {
            env
        }
    }

    /// Get the eval'd expression of `name`.
    pub fn get_name(env: Rc<RefCell<Env>>, cont: Rc<RefCell<Cont>>, name: &str) -> Expr {
        if !env.borrow().names.contains_key(name) {
            if !env.borrow().vars.contains_key(name) {
                match env.borrow().outer.clone() {
                    None => panic!("Woland: `{}` is not a defined (mutable) name.", name),
                    Some(oenv) => Self::get_name(oenv, cont, name),
                }
            } else {
                env.borrow().vars.get(name).unwrap().clone()
            }
        } else {
            env.borrow().names.get(name).unwrap().clone()
        }
    }
}

impl Expr {
    pub fn eval(&self, env: Rc<RefCell<Env>>, cont: Rc<RefCell<Cont>>) -> Expr {
        match self {
            Expr::Void => Expr::Void,
            Expr::I64(i) => Expr::I64(*i),
            Expr::Bool(b) => Expr::Bool(*b),
            Expr::Str(s) => Expr::Str(s.to_string()),
            Expr::List(l) => Expr::List(match l {
                List::Nil => List::Nil,
                List::Cons(h, t) => List::Cons(
                    Box::new(h.eval(env.clone(), cont.clone())),
                    Box::new(t.eval(env, cont)),
                ),
            }),
            Expr::Name(id) => Env::get_name(env, cont, id),
            Expr::Func {
                ann, param, body, ..
            } => Expr::Func {
                ann: ann.clone(),
                param: param.clone(),
                body: body.clone(),
                // Evaluating a function expression amounts to
                // capturing the current Env for future reference.
                closure: env,
            },
            Expr::Branch { paths } => {
                let mut result = Expr::Void;
                for p in paths {
                    if let Expr::Bool(b) = p.0.eval(env.clone(), cont.clone()) {
                        if b {
                            let (last, init) = p.1.split_last().unwrap();
                            for i in init {
                                i.execute(env.clone(), cont.clone());
                            }
                            result = last.execute(env, cont);
                            break;
                        }
                    } else {
                        unreachable!()
                    }
                }
                result
            }

            Expr::Block { body } => {
                // Evaluating a block needs a seperate Env.
                let benv = Rc::new(RefCell::new(Env::default()));
                benv.borrow_mut().outer = Some(env);
                let (last, init) = body.split_last().unwrap();
                for i in init {
                    i.execute(benv.clone(), cont.clone());
                }
                last.execute(benv, cont)
            }
            Expr::Apply { left, right } => {
                if let Expr::Func {
                    param,
                    body,
                    closure,
                    ..
                } = left.eval(env.clone(), cont.clone())
                {
                    // Evaluating a function-block needs a seperate Env
                    // The current env is only needed for resolving the parameter,
                    // which is inserted in the function's private Env alongside
                    // all its local definitions. Any other "external" names are
                    // reslved with the closure Env saved upon the evaluation
                    // of the Function expression. This might by the Env of another
                    // function application or a block expression.
                    let fenv = Rc::new(RefCell::new(Env::default()));
                    let rhs = right.eval(env, cont.clone());
                    fenv.borrow_mut().names.insert(param, rhs);
                    fenv.borrow_mut().outer = Some(closure);
                    // NOTE: the parser should've already ensured
                    // the body is not empty, so unwrap away!
                    let (last, init) = body.split_last().unwrap();
                    for instr in init {
                        instr.execute(fenv.clone(), cont.clone());
                    }
                    last.execute(fenv, cont)
                } else {
                    // TODO: switch all unreachable!'s to the unreachable
                    // intrinsic for more optimzation (?)
                    unreachable!()
                }
            }
            Expr::Intrinsic { name, args } => match name.as_str() {
                "dump" => {
                    for a in args {
                        print!("{}", a.eval(env.clone(), cont.clone()))
                    }
                    println!();
                    Expr::Void
                }
                "read" => {
                    let mut buffer = String::new();
                    io::stdin()
                        .read_to_string(&mut buffer)
                        .expect("Woland: error reading from stdin. You are on your own.");
                    Expr::Str(buffer)
                }
                "cmp" => {
                    Expr::Bool(args[0].eval(env.clone(), cont.clone()) == args[1].eval(env, cont))
                }
                "add" => {
                    if let Expr::I64(l) = args[0].eval(env.clone(), cont.clone()) {
                        if let Expr::I64(r) = args[1].eval(env, cont) {
                            Expr::I64(l + r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "sub" => {
                    if let Expr::I64(l) = args[0].eval(env.clone(), cont.clone()) {
                        if let Expr::I64(r) = args[1].eval(env, cont) {
                            Expr::I64(l - r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "mul" => {
                    if let Expr::I64(l) = args[0].eval(env.clone(), cont.clone()) {
                        if let Expr::I64(r) = args[1].eval(env, cont) {
                            Expr::I64(l * r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "div" => {
                    if let Expr::I64(l) = args[0].eval(env.clone(), cont.clone()) {
                        if let Expr::I64(r) = args[1].eval(env, cont) {
                            Expr::I64(l / r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "mod" => {
                    if let Expr::I64(l) = args[0].eval(env.clone(), cont.clone()) {
                        if let Expr::I64(r) = args[1].eval(env, cont) {
                            Expr::I64(l % r)
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                "clj" => {
                    for a in args {
                        if let Expr::Func { closure, .. } = a.eval(env.clone(), cont.clone()) {
                            println!("{:#?}", closure);
                        }
                    }
                    Expr::Void
                }
                "cons" => {
                    if let Expr::List(l) = args[1].eval(env.clone(), cont.clone()) {
                        Expr::List(List::Cons(
                            Box::new(args[0].eval(env.clone(), cont.clone())),
                            Box::new(Expr::List(l)),
                        ))
                    } else {
                        panic!("Woland: can only get the head of a list.");
                    }
                }
                "head" => {
                    if let Expr::List(l) = args[0].eval(env.clone(), cont.clone()) {
                        match l {
                            List::Nil => panic!("Woland: head: empty list."),
                            List::Cons(h, _) => *h,
                        }
                    } else {
                        panic!("Woland: can only get the head of a list.");
                    }
                }
                "tail" => {
                    if let Expr::List(l) = args[0].eval(env.clone(), cont.clone()) {
                        match l {
                            List::Nil => panic!("Woland: head: empty list."),
                            List::Cons(_, t) => *t,
                        }
                    } else {
                        panic!("Woland: can only get the tail of a list.");
                    }
                }
                _ => Expr::Void,
            },
        }
    }
}

impl Instr {
    fn execute(&self, env: Rc<RefCell<Env>>, cont: Rc<RefCell<Cont>>) -> Expr {
        match self {
            Instr::Compute(expr) => {
                // The evaluated expression may or may not have
                // any side-effects. Beware!
                expr.eval(env, cont)
            }
            Instr::Var { name, expr, .. } => {
                // NOTE: evaluation of the RHS expr should be seperate from
                // the insertion. Otherwise one would get a BorrowError at
                // runtime! i.e we shouldn't be needlessly holding a borrow
                // from eval when we try to get a mutable borrow of the Env.
                let rhs = expr.eval(env.clone(), cont);
                env.borrow_mut().vars.insert(name.clone(), rhs);
                Expr::Void
            }
            Instr::Assign { name, expr, .. } => {
                let venv = Env::get_var_env(env.clone(), name);
                // Imperative assignment enforces strict evalation,
                // otherwise we can't do simple Assign's such as
                // i = i + 1
                let rhs = expr.eval(env, cont);
                venv.borrow_mut().vars.insert(name.clone(), rhs);
                Expr::Void
            }
            Instr::Let(dname) => {
                let rhs = dname.expr.eval(env.clone(), cont);
                env.borrow_mut().names.insert(dname.name.clone(), rhs);
                Expr::Void
            }
            Instr::Loop { body } => {
                // The loops variables keeps track of the level of
                // nested loops we reached. Hence as long as its value
                // doesn't change (through a break instruction) we can
                // keep executing the current block. Nested loops work
                // by replicating this behviour one level up.
                // TODO: expand this by implementing `continue`.
                cont.borrow_mut().loops += 1;
                let start = cont.borrow().loops;
                while start == cont.borrow().loops {
                    for i in body {
                        i.execute(env.clone(), cont.clone());
                        if cont.borrow().loops != start {
                            break;
                        }
                    }
                }
                Expr::Void
            }
            Instr::Break => {
                if cont.borrow().loops == 0 {
                    panic!("Woland: can only break out of a loop.")
                }
                cont.borrow_mut().loops -= 1;
                Expr::Void
            }
            Instr::Ellipsis => {
                // Do nothing! This is a simple filler Instr
                Expr::Void
            }
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::I64(i) => write!(f, "{}", i),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::Str(s) => write!(f, "{}", s),
            other => write!(f, "{:#?}", other),
        }
    }
}

#[cfg(test)]
mod tests {}

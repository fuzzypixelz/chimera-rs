use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use crate::ast::{DName, Expr, Instr};
use crate::code::{Code, CompiledCode, Env};
use crate::value::{List, Value};

impl<'c> Code<'c> for Expr {
    fn compile(self) -> CompiledCode<'c> {
        match self {
            Expr::Void => CompiledCode::new(move |_env, _cont| Rc::new(Value::Void)),
            Expr::Int(int) => {
                CompiledCode::new(move |_env, _cont| Rc::new(Value::Int(int)))
            }
            Expr::Bool(boolean) => {
                CompiledCode::new(move |_env, _cont| Rc::new(Value::Bool(boolean)))
            }
            Expr::Str(string) => {
                CompiledCode::new(move |_env, _cont| Rc::new(Value::Str(string.clone())))
            }
            Expr::List(list) => {
                let compiled_list =
                    list.into_iter().map(|i| i.compile()).collect::<Vec<_>>();
                CompiledCode::new(move |env, cont| {
                    Rc::new(Value::List({
                        compiled_list
                            .iter()
                            .map(|i| i.execute(env.clone(), cont.clone()))
                            .collect::<Vec<_>>()
                            .into()
                    }))
                })
            }
            Expr::Name(name) => {
                CompiledCode::new(move |env, _cont| Env::get_name(env, &name))
            }
            Expr::Block { mut body } => {
                // let (last, init) = body.split_last().unwrap();
                let last = body.pop().unwrap();
                let compiled_block =
                    body.into_iter().map(|i| i.compile()).collect::<Vec<_>>();
                let compiled_expr = last.compile();
                CompiledCode::new(move |env, cont| {
                    for instr in compiled_block.iter() {
                        instr.execute(env.clone(), cont.clone());
                    }
                    compiled_expr.execute(env, cont)
                })
            }
            Expr::Branch { paths } => {
                let compiled_branch = paths
                    .into_iter()
                    .map(|(c, b)| {
                        (
                            c.compile(),
                            b.into_iter().map(|i| i.compile()).collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                CompiledCode::new(move |env, cont| {
                    let mut result = Rc::new(Value::Void);
                    for p in &compiled_branch {
                        if let Value::Bool(b) =
                            *p.0.execute(env.clone(), cont.clone()).clone()
                        {
                            if b {
                                let (last, init) = p.1.split_last().unwrap();
                                // let mut body = p.1.clone();
                                // let last = body.pop().unwrap();
                                // let compiled_block =
                                // body.into_iter().map(|i| i.compile()); //.collect::<Vec<_>>();
                                // let compiled_expr = last.compile();
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
                })
            }
            Expr::Func { param, body, .. } => {
                let compiled_body =
                    Rc::new(body.into_iter().map(Code::compile).collect::<Vec<_>>());
                CompiledCode::new(move |env, _cont| {
                    Rc::new(Value::Func {
                        param: param.clone(),
                        body: compiled_body.clone(),
                        // Evaluating a function expression amounts to
                        // capturing the current Env for future reference.
                        closure: env,
                    })
                })
            }
            Expr::Apply { left, right } => {
                let compiled_func = left.compile();
                let compiled_input = right.compile();
                CompiledCode::new(move |env, cont| {
                    if let Value::Func {
                        param,
                        body,
                        closure,
                        ..
                    } = &*compiled_func.execute(env.clone(), cont.clone())
                    {
                        // Evaluating a function-block needs a seperate Env
                        // The current env is only needed for resolving the parameter,
                        // which is inserted in the function's private Env alongside
                        // all its local definitions. Any other "external" names are
                        // reslved with the closure Env saved upon the evaluation
                        // of the Function expression. This might by the Env of another
                        // function application or a block expression.
                        let fenv = Rc::new(RefCell::new(Env::default()));
                        let input_value =
                            compiled_input.execute(env.clone(), cont.clone());
                        fenv.borrow_mut()
                            .names
                            .insert(param.to_string(), input_value);
                        fenv.borrow_mut().outer = Some(closure.clone());
                        // NOTE: the parser should've already ensured
                        // the body is not empty, so unwrap away!
                        let (last, init) = body.split_last().unwrap();
                        for i in init.iter() {
                            i.execute(fenv.clone(), cont.clone());
                        }
                        last.execute(fenv, cont)
                    } else {
                        // TODO: switch all unreachable!'s to the unreachable
                        // intrinsic for more optimzation (?)
                        unreachable!()
                    }
                })
            }
            Expr::Intrinsic { name, args } => {
                let compiled_args =
                    args.into_iter().map(|i| i.compile()).collect::<Vec<_>>();
                match name.as_str() {
                    "dump" => CompiledCode::new(move |env, cont| {
                        for a in compiled_args.iter() {
                            print!("{}", a.execute(env.clone(), cont.clone()))
                        }
                        println!();
                        Rc::new(Value::Void)
                    }),
                    "read" => CompiledCode::new(move |_env, _cont| {
                        let mut buffer = String::new();
                        io::stdin().read_to_string(&mut buffer).expect(
                            "Woland: error reading from stdin. You are on your own.",
                        );
                        Rc::new(Value::Str(buffer))
                    }),
                    "cmp" => CompiledCode::new(move |env, cont| {
                        Rc::new(Value::Bool(
                            compiled_args[0].execute(env.clone(), cont.clone())
                                == compiled_args[1].execute(env, cont),
                        ))
                    }),
                    "add" => CompiledCode::new(move |env, cont| {
                        if let Value::Int(l) =
                            *compiled_args[0].execute(env.clone(), cont.clone())
                        {
                            if let Value::Int(r) = *compiled_args[1].execute(env, cont) {
                                Rc::new(Value::Int(l + r))
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }),
                    "sub" => CompiledCode::new(move |env, cont| {
                        if let Value::Int(l) =
                            *compiled_args[0].execute(env.clone(), cont.clone())
                        {
                            if let Value::Int(r) = *compiled_args[1].execute(env, cont) {
                                Rc::new(Value::Int(l - r))
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }),
                    "mul" => CompiledCode::new(move |env, cont| {
                        if let Value::Int(l) =
                            *compiled_args[0].execute(env.clone(), cont.clone())
                        {
                            if let Value::Int(r) = *compiled_args[1].execute(env, cont) {
                                Rc::new(Value::Int(l * r))
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }),
                    "div" => CompiledCode::new(move |env, cont| {
                        if let Value::Int(l) =
                            *compiled_args[0].execute(env.clone(), cont.clone())
                        {
                            if let Value::Int(r) = *compiled_args[1].execute(env, cont) {
                                Rc::new(Value::Int(l / r))
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }),
                    "mod" => CompiledCode::new(move |env, cont| {
                        if let Value::Int(l) =
                            *compiled_args[0].execute(env.clone(), cont.clone())
                        {
                            if let Value::Int(r) = *compiled_args[1].execute(env, cont) {
                                Rc::new(Value::Int(l % r))
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }),
                    "cons" => CompiledCode::new(move |env, cont| {
                        if let Value::List(l) =
                            &*compiled_args[1].execute(env.clone(), cont.clone())
                        {
                            Rc::new(Value::List(List::Cons(
                                compiled_args[0].execute(env, cont),
                                Box::new(l.clone()),
                            )))
                        } else {
                            panic!("Woland: can only call cons on a list.");
                        }
                    }),
                    "head" => CompiledCode::new(move |env, cont| {
                        if let Value::List(l) = &*compiled_args[0].execute(env, cont) {
                            match l {
                                List::Nil => panic!("Woland: head: empty list."),
                                List::Cons(h, _) => h.clone(),
                            }
                        } else {
                            panic!("Woland: can only get the head of a list.");
                        }
                    }),
                    "tail" => CompiledCode::new(move |env, cont| {
                        if let Value::List(l) = &*compiled_args[0].execute(env, cont) {
                            match l {
                                List::Nil => panic!("Woland: head: empty list."),
                                List::Cons(_, t) => Rc::new(Value::List(*t.clone())),
                            }
                        } else {
                            panic!("Woland: can only get the tail of a list.");
                        }
                    }),
                    _ => CompiledCode::default(),
                }
            }
        }
    }
}

impl<'c> Code<'c> for Instr {
    fn compile(self) -> CompiledCode<'c> {
        match self {
            Instr::Compute(expr) => {
                let compiled_expr = expr.compile();
                // The evaluated expression may or may not have
                // any side-effects. Beware!
                CompiledCode::new(move |env, cont| compiled_expr.execute(env, cont))
            }
            Instr::Var { name, expr, .. } => {
                let compiled_expr = expr.compile();
                CompiledCode::new(move |env, cont| {
                    // NOTE: evaluation of the RHS expr should be seperate from
                    // the insertion. Otherwise one would get a BorrowError at
                    // runtime! i.e we shouldn't be needlessly holding a borrow
                    // from eval when we try to get a mutable borrow of the Env.
                    let rhs_value = compiled_expr.execute(env.clone(), cont);
                    env.borrow_mut().vars.insert(name.to_string(), rhs_value);
                    Rc::new(Value::Void)
                })
            }
            Instr::Assign { name, expr, .. } => {
                let compiled_expr = expr.compile();
                CompiledCode::new(move |env, cont| {
                    // Imperative assignment enforces strict evalation,
                    // otherwise we can't do simple Assign's such as
                    // i = i + 1
                    let venv = Env::get_var_env(env.clone(), &name);
                    let rhs_value = compiled_expr.execute(env, cont);
                    venv.borrow_mut().vars.insert(name.to_string(), rhs_value);
                    Rc::new(Value::Void)
                })
            }
            Instr::Let(DName { name, expr, .. }) => {
                let compiled_expr = expr.compile();
                CompiledCode::new(move |env, cont| {
                    let rhs_value = compiled_expr.execute(env.clone(), cont);
                    env.borrow_mut().names.insert(name.to_string(), rhs_value);
                    Rc::new(Value::Void)
                })
            }
            Instr::Loop { body } => {
                let compiled_block =
                    body.into_iter().map(|i| i.compile()).collect::<Vec<_>>();
                CompiledCode::new(move |env, cont| {
                    // The loops variables keeps track of the level of
                    // nested loops we reached. Hence as long as its value
                    // doesn't change (through a break instruction) we can
                    // keep executing the current block. Nested loops work
                    // by replicating this behviour one level up.
                    // TODO: expand this by implementing `continue`.
                    cont.borrow_mut().loops += 1;
                    let start = cont.borrow().loops;
                    while start == cont.borrow().loops {
                        for i in compiled_block.iter() {
                            i.execute(env.clone(), cont.clone());
                            if cont.borrow().loops != start {
                                break;
                            }
                        }
                    }
                    Rc::new(Value::Void)
                })
            }
            Instr::Break => CompiledCode::new(move |_env, cont| {
                if cont.borrow().loops == 0 {
                    panic!("Woland: can only break out of a loop.")
                }
                cont.borrow_mut().loops -= 1;
                Rc::new(Value::Void)
            }),
            Instr::Ellipsis => {
                // Do nothing! This is a simple filler Instr
                CompiledCode::default()
            }
        }
    }
}

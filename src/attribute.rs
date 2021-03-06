use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;

use crate::code::CompiledCode;
use crate::value::{List, Value, WoValue};
use crate::Env;

pub fn intrinsic(name: &str) -> Value {
    match name {
        "print" => Value::Lambda {
            param: "value".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                let value = Env::get_name(env, "value");
                print!("{}", value.borrow());
                Value::Void.into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "read" => Value::Lambda {
            param: "_".to_string(),
            body: Rc::new(CompiledCode::new(move |_env| {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .expect("chimera: error reading from stdin. You are on your own.");
                let string = buffer
                    .chars()
                    .map(|c| Value::Char(c).into())
                    .collect::<Vec<WoValue>>()
                    .into();
                Value::List(string).into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "cmp" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        let x = Env::get_name(env.clone(), "x");
                        let y = Env::get_name(env, "y");
                        Value::Bool(x == y).into()
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "add" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        if let Value::Int(l) = *Env::get_name(env.clone(), "x").borrow() {
                            if let Value::Int(r) = *Env::get_name(env, "y").borrow() {
                                Value::Int(l + r).into()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "sub" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        if let Value::Int(l) = *Env::get_name(env.clone(), "x").borrow() {
                            if let Value::Int(r) = *Env::get_name(env, "y").borrow() {
                                Value::Int(l - r).into()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "mul" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        if let Value::Int(l) = *Env::get_name(env.clone(), "x").borrow() {
                            if let Value::Int(r) = *Env::get_name(env, "y").borrow() {
                                Value::Int(l * r).into()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "div" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        if let Value::Int(l) = *Env::get_name(env.clone(), "x").borrow() {
                            if let Value::Int(r) = *Env::get_name(env, "y").borrow() {
                                Value::Int(l / r).into()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "modulus" => Value::Lambda {
            param: "x".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "y".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        if let Value::Int(l) = *Env::get_name(env.clone(), "x").borrow() {
                            if let Value::Int(r) = *Env::get_name(env, "y").borrow() {
                                Value::Int(l % r).into()
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "cons" => Value::Lambda {
            param: "elem".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                Value::Lambda {
                    param: "list".to_string(),
                    body: Rc::new(CompiledCode::new(move |env| {
                        let elem = Env::get_name(env.clone(), "elem");
                        if let Value::List(list) = &*Env::get_name(env, "list").borrow() {
                            Value::List(List::Cons(elem, Box::new(list.clone()))).into()
                        } else {
                            panic!("chimera: can only call cons on a list.");
                        }
                    })),
                    closure: env,
                }
                .into()
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "head" => Value::Lambda {
            param: "list".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                if let Value::List(list) = &*Env::get_name(env, "list").borrow() {
                    match list {
                        List::Nil => panic!("chimera: head: empty list."),
                        List::Cons(h, _) => h.clone(),
                    }
                } else {
                    panic!("chimera: can only get the head of a list.");
                }
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        "tail" => Value::Lambda {
            param: "list".to_string(),
            body: Rc::new(CompiledCode::new(move |env| {
                if let Value::List(list) = &*Env::get_name(env, "list").borrow() {
                    match list {
                        List::Nil => panic!("chimera: tail: empty list."),
                        List::Cons(_, t) => Value::List(*t.clone()).into(),
                    }
                } else {
                    panic!("chimera: can only get the tail of a list.");
                }
            })),
            closure: Rc::new(RefCell::new(Env::default())),
        },
        _ => panic!("chimera: unknown intrinsic attribute {}", name),
    }
}


mod attribute;
pub mod code;
mod value;

use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{Expr, Item, ItemKind, Stmt};
use attribute::intrinsic;
use code::{Code, CompiledCode, Env};
use value::Value;

impl Code for Expr {
    fn compile(self) -> CompiledCode {
        match self {
            Expr::Ellipsis => CompiledCode::new(move |_env| Value::Void.into()),
            Expr::Void => CompiledCode::new(move |_env| Value::Void.into()),
            Expr::Int(int) => CompiledCode::new(move |_env| Value::Int(int).into()),
            Expr::Bool(boolean) => CompiledCode::new(move |_env| Value::Bool(boolean).into()),
            Expr::Char(ch) => CompiledCode::new(move |_env| Value::Char(ch).into()),
            Expr::Name(name) => CompiledCode::new(move |env| Env::get_name(env, &name)),
            Expr::List(list) => {
                let compiled_list = list.into_iter().map(Code::compile).collect::<Vec<_>>();
                CompiledCode::new(move |env| {
                    Rc::new(RefCell::new(Value::List({
                        compiled_list
                            .iter()
                            .map(|i| i.execute(env.clone()))
                            .collect::<Vec<_>>()
                            .into()
                    })))
                })
            }
            Expr::Block { mut body } => {
                // NOTE: the parser should've already ensured
                // the body is not empty, so unwrap away!
                let last = body.pop().unwrap();
                let compiled_block = body.into_iter().map(Code::compile).collect::<Vec<_>>();
                let compiled_expr = last.compile();
                CompiledCode::new(move |env| {
                    for instr in compiled_block.iter() {
                        instr.execute(env.clone());
                    }
                    compiled_expr.execute(env)
                })
            }
            Expr::Branch { paths } => {
                let compiled_branch = paths
                    .into_iter()
                    .map(|(c, b)| {
                        (
                            c.compile(),
                            b.into_iter().map(Code::compile).collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                CompiledCode::new(move |env| {
                    let mut result = Value::Void.into();
                    for p in &compiled_branch {
                        // FIXME: it's not very clear that p.0 is the condition and
                        // p.1 the corresponding code.
                        if let Value::Bool(b) = *p.0.execute(env.clone()).borrow() {
                            if b {
                                let (last, init) = p.1.split_last().unwrap();
                                for i in init {
                                    i.execute(env.clone());
                                }
                                result = last.execute(env);
                                break;
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    result
                })
            }
            Expr::Lambda { param, expr } => {
                let compiled_body = Rc::new(expr.compile());
                CompiledCode::new(move |env| {
                    Value::Lambda {
                        param: param.clone(),
                        // The function's body is compiled the first time we come
                        // across its expression, then its expression itself
                        // is transformed into a closure that remembers this
                        // compiled block.
                        // The .clone() is necessary because we are moving the
                        // compiled body out of the closure (as a Value, functions
                        // are first-class), but it would be too slow to duplicate
                        // the Vec<CompiledCode> each time we want to use, so
                        // it is fitting to wrap it in an Rc as it's read-only.
                        body: compiled_body.clone(),
                        // Evaluating a function expression has the effect of
                        // capturing the current Env for future reference.
                        closure: env,
                    }
                    .into()
                })
            }
            Expr::Apply { left, right } => {
                let compiled_func = left.compile();
                let compiled_input = right.compile();
                CompiledCode::new(move |env| {
                    if let Value::Lambda {
                        param,
                        body,
                        closure,
                        ..
                    } = &*compiled_func.execute(env.clone()).borrow()
                    {
                        // Evaluating a function-block needs a separate Env
                        // The current env is only needed for resolving the parameter,
                        // which is inserted in the function's private Env alongside
                        // all its local definitions. Any other "external" names are
                        // resolved with the closure Env saved upon the evaluation
                        // of the Function expression. This might by the Env of another
                        // function application or a block expression.
                        let fenv = Rc::new(RefCell::new(Env::default()));
                        let input_value = compiled_input.execute(env);
                        fenv.borrow_mut()
                            .names
                            .insert(param.to_string(), input_value);
                        fenv.borrow_mut().outer = Some(closure.clone());
                        body.execute(fenv)
                    } else {
                        // TODO: switch all unreachable!'s to the unreachable
                        // intrinsic for more optimization (?)
                        unreachable!()
                    }
                })
            }
            _ => unimplemented!("expression {self:?} is not evaluated!"),
        }
    }
}

impl Code for Stmt {
    fn compile(self) -> CompiledCode {
        match self {
            Stmt::Expr(expr) => {
                let compiled_expr = expr.compile();
                // The evaluated expression may or may not have
                // any side-effects. Beware!
                CompiledCode::new(move |env| compiled_expr.execute(env))
            }
            Stmt::Item(item) => {
                let compiled_item = item.compile();
                CompiledCode::new(move |env| compiled_item.execute(env))
            }
        }
    }
}

impl Code for Item {
    fn compile(self) -> CompiledCode {
        match self.kind {
            ItemKind::Definition { name, expr, .. } => {
                let compiled_expr = expr.compile();
                match self.attr {
                    None => CompiledCode::new(move |env| {
                        let rhs_value = compiled_expr.execute(env.clone());
                        env.borrow_mut().names.insert(name.to_string(), rhs_value);
                        Value::Void.into()
                    }),
                    Some(attr) => {
                        if attr.name == "intrinsic" {
                            CompiledCode::new(move |env| {
                                let rhs_value = intrinsic(&attr.args[0]).into();
                                env.borrow_mut().names.insert(name.to_string(), rhs_value);
                                Value::Void.into()
                            })
                        } else {
                            panic!("chimera: unknown attribute {}", attr.name)
                        }
                    }
                }
            }
            _ => unimplemented!("item {self:#?} is not evaluated!"),
        }
    }
}

use fnv::FnvHashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

pub trait Code<'c> {
    fn compile(self) -> CompiledCode<'c>;
}

pub struct CompiledCode<'c>(
    Box<dyn 'c + Fn(Rc<RefCell<Env>>, Rc<RefCell<Cont>>) -> Rc<Value>>,
);

impl<'c> CompiledCode<'c> {
    pub fn new(
        closure: impl 'c + Fn(Rc<RefCell<Env>>, Rc<RefCell<Cont>>) -> Rc<Value>,
    ) -> Self {
        Self(Box::new(closure))
    }

    pub fn execute(
        &self,
        env: Rc<RefCell<Env<'c>>>,
        cont: Rc<RefCell<Cont>>,
    ) -> Rc<Value<'c>> {
        self.0(env, cont)
    }
}

impl<'c> Default for CompiledCode<'c> {
    fn default() -> Self {
        CompiledCode::new(|_env, _cont| Rc::new(Value::Void))
    }
}

// A "Continuation" i.e the evaluator's state.
#[derive(Clone)]
pub struct Cont {
    /// Keeps track of the number of nested loops.
    pub loops: u64,
}

impl Cont {
    pub fn default() -> Self {
        Self { loops: 0 }
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct Env<'c> {
    pub names: FnvHashMap<String, Rc<Value<'c>>>,
    pub vars: FnvHashMap<String, Rc<Value<'c>>>,
    pub outer: Option<Rc<RefCell<Env<'c>>>>,
}

impl<'c> Env<'c> {
    /// Get the first Env containing `name`, this was we can mutate the var directly.
    pub fn get_var_env(env: Rc<RefCell<Env<'c>>>, name: &str) -> Rc<RefCell<Env<'c>>> {
        if !env.borrow().vars.contains_key(name) {
            match env.borrow().outer.clone() {
                None => {
                    panic!("Woland: `{}` is not a defined mutable name.", name)
                }
                Some(oenv) => Self::get_var_env(oenv, name),
            }
        } else {
            env
        }
    }

    /// Get the eval'd expression of `name`.
    pub fn get_name(env: Rc<RefCell<Env<'c>>>, name: &str) -> Rc<Value<'c>> {
        if !env.borrow().names.contains_key(name) {
            if !env.borrow().vars.contains_key(name) {
                match env.borrow().outer.clone() {
                    None => panic!("Woland: `{}` is not a defined (mutable) name.", name),
                    Some(oenv) => Self::get_name(oenv, name),
                }
            } else {
                env.borrow().vars.get(name).unwrap().clone()
            }
        } else {
            env.borrow().names.get(name).unwrap().clone()
        }
    }
}

use fnv::FnvHashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::value::WoValue;

/// A data-type is `Code` if it can produce a function from `(Env, Cont)` to `Value`,
/// this Fn is used to (sort of) JIT compile `Expr`'s and `Instr`'s to reusable bits.
pub trait Code {
    fn compile(self) -> CompiledCode;
}

pub struct CompiledCode(Box<dyn Fn(WoEnv, WoCont) -> WoValue>);

impl CompiledCode {
    pub fn new(closure: impl 'static + Fn(WoEnv, WoCont) -> WoValue) -> Self {
        Self(Box::new(closure))
    }

    pub fn execute(&self, env: WoEnv, cont: WoCont) -> WoValue {
        self.0(env, cont)
    }
}

impl Default for CompiledCode {
    fn default() -> Self {
        CompiledCode::new(|_env, _cont| WoValue::default())
    }
}

impl std::fmt::Debug for CompiledCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Cannot debug-print closures.
        write!(f, "[CompiledCode]")
    }
}

pub type WoCont = Rc<RefCell<Cont>>;

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

pub type WoEnv = Rc<RefCell<Env>>;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Env {
    pub names: FnvHashMap<String, WoValue>,
    pub vars: FnvHashMap<String, WoValue>,
    pub outer: Option<WoEnv>,
}

impl Env {
    /// Get the first Env containing `name`, this was we can mutate the var directly.
    pub fn get_var_env(env: WoEnv, name: &str) -> WoEnv {
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
    pub fn get_name(env: WoEnv, name: &str) -> WoValue {
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

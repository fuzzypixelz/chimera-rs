use std::cell::RefCell;
use std::rc::Rc;

use fnv::FnvHashMap;

use super::value::WoValue;

/// A data-type is `Code` if it can produce a function from `(Env, Cont)` to `Value`,
/// this Fn is used to (sort of) JIT compile `Expr`'s and `Instr`'s to reusable bits.
/// This means the compiler will tranverse the AST recursively and only call `compile()`
/// on a piece of `Code` if it needs to evaluate it, each time producing a new closure.
/// For example, the function application `addOne 41` will evaluate the following way:
///     1. The name expression `(+)` is transformed into an Fn(Env) that queries
///        `Env` for the value of `(+)` which itself is a Value::Func { .. } containing
///        a Fn(Env) that take a _function env_ and produces the result of adding 1 to
///        its parameter.
///     2. The literal expression `1` is transformed into a Fn(Env) that discard its env
///        and simply returns the Value `1`.
///     3. Now that both the left and right expressions have been JIT compiled, we can
///        evaluate `addOne 41` by inserting the Value `1` into the _function env_ and
///        executing `addOne`'s compiled code on this Env.
pub trait Code {
    fn compile(self) -> CompiledCode;
}

// FIXME: This is a bad model of `Code` as Instr's are not supposed
// to have values nor types, it would make more sense to return am
// optional WoValue, but would it cause more overhead to check it?
pub struct CompiledCode(Box<dyn Fn(WoEnv) -> WoValue>);

impl CompiledCode {
    pub fn new(closure: impl 'static + Fn(WoEnv) -> WoValue) -> Self {
        Self(Box::new(closure))
    }

    pub fn execute(&self, env: WoEnv) -> WoValue {
        self.0(env)
    }
}

impl Default for CompiledCode {
    fn default() -> Self {
        CompiledCode::new(|_env| WoValue::default())
    }
}

impl std::fmt::Debug for CompiledCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Cannot debug-print closures.
        write!(f, "[CompiledCode]")
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
    /// Get the first `Env` containing `name`.
    pub fn get_var_env(env: WoEnv, name: &str) -> WoEnv {
        if !env.borrow().vars.contains_key(name) {
            match env.borrow().outer.clone() {
                None => {
                    panic!("chimera: `{}` is not a defined mutable name.", name)
                }
                Some(oenv) => Self::get_var_env(oenv, name),
            }
        } else {
            env
        }
    }

    /// Get the value corresponding to `name` in the chain of `Env`s.
    pub fn get_name(env: WoEnv, name: &str) -> WoValue {
        if !env.borrow().names.contains_key(name) {
            if !env.borrow().vars.contains_key(name) {
                match env.borrow().outer.clone() {
                    // TODO: This panic is no longer necessary as the typechecker
                    // is supposed to catch them beforehand. The same goes for all
                    // the `if let ... {} else {}` blocks in interpreter
                    // It would make more sense to do an `unreachable_unchecked()`
                    // in this case. That function however is unsafe and complete UB.
                    // I would argue it's acceptable since Algorithm J is mathematically
                    // proven to always correcly infer the most general type for an expr,
                    // of course, I can't same say the same for my implementation of it :^).
                    None => panic!("chimera: `{}` is not a defined (mutable) name.", name),
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

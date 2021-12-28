use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::code::{CompiledCode, Env};

#[derive(PartialEq)]
pub enum Value<'c> {
    Void,
    Int(i64),
    Bool(bool),
    Str(String),
    List(List<'c>),
    Func {
        param: String,
        body: Rc<Vec<CompiledCode<'c>>>,
        closure: Rc<RefCell<Env<'c>>>,
    },
}

impl<'c> PartialEq for CompiledCode<'c> {
    fn eq(&self, _other: &Self) -> bool {
        // We don't provide comparison of
        // functions at the language level.
        false
    }
}

/// The representation of a "Cons List" within the interpreter,
/// as the language isn't mature enough to have custom data types yet.
/// This is a temporary way of having aggregate data types in Woland.
#[derive(Clone, PartialEq)]
pub enum List<'c> {
    Cons(Rc<Value<'c>>, Box<List<'c>>),
    Nil,
}

impl<'c> From<Vec<Rc<Value<'c>>>> for List<'c> {
    fn from(mut item: Vec<Rc<Value<'c>>>) -> Self {
        // FIXME: this is too slow!
        if item.is_empty() {
            List::Nil
        } else {
            // Could be better written, probably.
            let tail = item.drain(1..).collect::<Vec<_>>();
            let head = item.pop().unwrap();
            List::Cons(head, Box::new(tail.into()))
        }
    }
}

impl<'c> Display for Value<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            _other => write!(f, "unimplemented"),
        }
    }
}

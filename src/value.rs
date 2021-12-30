use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::code::{CompiledCode, WoEnv};

pub type WoValue<'c> = Rc<RefCell<Value<'c>>>;

#[derive(Debug, PartialEq)]
pub enum Value<'c> {
    Void,
    Int(i64),
    Bool(bool),
    Char(char),
    Str(String),
    List(List<'c>),
    Array(Vec<WoValue<'c>>),
    Func {
        param: String,
        body: Rc<Vec<CompiledCode<'c>>>,
        closure: WoEnv<'c>,
    },
}

impl<'c> Default for Value<'c> {
    fn default() -> Self {
        Value::Void
    }
}

impl<'c> From<Value<'c>> for Rc<RefCell<Value<'c>>> {
    fn from(item: Value<'c>) -> Self {
        Rc::new(RefCell::new(item))
    }
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
#[derive(Debug, Clone, PartialEq)]
pub enum List<'c> {
    Cons(WoValue<'c>, Box<List<'c>>),
    Nil,
}

impl<'c> From<Vec<WoValue<'c>>> for List<'c> {
    fn from(mut item: Vec<WoValue<'c>>) -> Self {
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

// impl<'c> From<List<'c>> for Vec<WoValue<'c>> {
impl<'c> Into<Vec<WoValue<'c>>> for List<'c> {
    fn into(mut self) -> Vec<WoValue<'c>> {
        let mut result = Vec::new();
        loop {
            match self {
                List::Cons(v, l) => {
                    result.push(v);
                    self = *l;
                }
                List::Nil => break result,
            }
        }
    }
}

impl<'c> Display for Value<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "()"),
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::List(l) => write!(f, "[{}]", l),
            Value::Array(a) => write!(
                f,
                "#[{}]",
                a.iter()
                    .map(|v| v.borrow().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Func { .. } => write!(f, "{:#?}", self),
        }
    }
}

impl<'c> Display for List<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Into::<Vec<WoValue<'c>>>::into(self.to_owned())
                .iter()
                .map(|v| v.borrow().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_into_vec() {
        let list: Vec<WoValue> = List::Cons(Value::Int(1).into(), Box::new(List::Nil)).into();
        assert_eq!(list, vec![Value::Int(1).into()])
    }
}

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::code::{CompiledCode, WoEnv};

pub type WoValue = Rc<RefCell<Value>>;

#[derive(Debug, PartialEq)]
pub enum Value {
    Void,
    Int(i64),
    Bool(bool),
    Char(char),
    Str(String),
    List(List),
    Array(Vec<WoValue>),
    Func {
        param: String,
        body: Rc<CompiledCode>,
        closure: WoEnv,
    },
}

impl Default for Value {
    fn default() -> Self {
        Value::Void
    }
}

impl From<Value> for Rc<RefCell<Value>> {
    fn from(item: Value) -> Self {
        Rc::new(RefCell::new(item))
    }
}

impl PartialEq for CompiledCode {
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
pub enum List {
    Cons(WoValue, Box<List>),
    Nil,
}

impl From<Vec<WoValue>> for List {
    fn from(mut item: Vec<WoValue>) -> Self {
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

// impl From<List> for Vec<WoValue> {
impl Into<Vec<WoValue>> for List {
    fn into(mut self) -> Vec<WoValue> {
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

impl Display for Value {
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

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Into::<Vec<WoValue>>::into(self.to_owned())
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

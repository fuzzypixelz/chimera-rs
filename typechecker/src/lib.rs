/*
    This file describes the Woland Type System.

    In order to check if the program is correctly typed
    statically, we need a way to answer whether or not
    a give `Expr` is of type T. So we can make sure that
    function calls, Branches and Assigns get expressions
    of the correct type.

    To know if an Expr is of type T, we only need to check
    if that expression matches any of T's constructors!
    Hence, if

      type Division = Ok { num: I64 } | DivByZeroErr

    we know that `Ok 42` is of type Result, but `true`
    is not, as it's not within the list of constructors.

    For primitive types suck as I64, we check differently
    based on enum variants.

    An expression can either be a primitive literal; constructed
    through the underlying implementation, a variable reference,
    or a function application. These are the three cases where we
    need to check types.

    It follows that a type can be either a primitive type; hardcoded
    into the language implementation: I64, Bool, String, Void.
    (NOTE: some of these will be replaced by alternatives defined in
    the userland, once the type system is ready. Only I64 is sure to
    stay.)
    Moreover we get user-defined types through `type` decls, and lastly,
    an arrow type of the form `A -> B` representing a function. On top of this,
    we need to distinguish between Pure(A -> B) and Impure(A -> B) on the type
    level. For example:
      - Pure(I64) is a pure-function that evaluates to an I64.
      - Impure(I64) is a procedure that returns an I64.
      - Var(I64) is a variable of type I64.
      - Pure(I64 -> Bool) maybe a `even` function.
      - I64 -> Bool doesn't mean anything.
    TODO: Figure out in which direction this should be expanded to include
    types of the form `A1 -> A2 -> ... -> An`. The most obvious choice is
    to internally compile these into `A1 -> (A2 -> (... (...-> An)))` and
    provide appropriate syntax.
*/
mod error;

use std::collections::HashMap;
use std::ops::Deref;
use std::result::Result;

use common::*;

use crate::error::TypeError;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Primitive data types
    // handled by the implementation.
    Void,
    I64,
    Bool,
    String,
    // Type declared by the user, uses
    // information from the AST.
    New(HashMap<String, Constr>),
    // Type for single-argument, single-
    // result functions: T -> T
    Func(Box<Type>, Box<Type>),
    // Impure(Box<Type>, Box<Type>),
    // Type of a variable, can be any of
    // the above. The "return type" of a
    // Pure should never be this.
    // NOTE: this has been dropped.
    // Var(Box<Type>),
}

pub type Constr = HashMap<String, Type>;

#[derive(Debug, Clone)]
pub struct Ctx {
    dtypes: HashMap<String, DType>,
    names: HashMap<String, Type>,
    vars: HashMap<String, Type>,
}

impl Ctx {
    pub fn new(prog: &AST) -> Self {
        let mut ctx = Ctx {
            dtypes: HashMap::new(),
            names: HashMap::new(),
            vars: HashMap::new(),
        };
        // for d in &prog.decls {
        //     if let Decl::Func(dfunc) = d {
        //         ctx.names.insert(
        //             dfunc.name.to_string(),
        //             Self::type_from_any(&dfunc.ann, &prog.decls),
        //         );
        //     }
        // }
        for d in &prog.decls {
            match d {
                Decl::Func(dfunc) => {
                    ctx.names.insert(
                        dfunc.name.to_string(),
                        ctx.type_from_any(&dfunc.ann));
                },
                Decl::Type(dtype) => {
                    ctx.dtypes.insert(
                        dtype.name.to_string(),
                        dtype.to_owned());
                }
            }
        }
        ctx
    }

    // Types of 0-arity functions
    fn type_from_one(&self, ann: &String) -> Type {
        match ann.as_ref() {
            "Void" => Type::Void,
            "I64" => Type::I64,
            "Bool" => Type::Bool,
            "String" => Type::String,
            _ => self.type_from_other(ann),
        }
    }

    // User defined types
    fn type_from_other(&self, ann: &String) -> Type {
        let dconstrs = &self
            .dtypes
            .get(ann)
            .unwrap() // TODO: handle this correctly.
            .body;
        let mut constrs = HashMap::new();
        for dc in dconstrs {
            let mut c = HashMap::new();
            if let Some(r) = &dc.record {
                for (name, ann) in r {
                    c.insert(name.to_string(), self.type_from_any(ann));
                }
            }
            constrs.insert(dc.name.to_string(), c);
        }
        Type::New(constrs)
    }

    fn type_from_any(&self, ann: &[String]) -> Type {
        if ann.len() == 1 {
            self.type_from_one(&ann[0])
        } else {
            Type::Func(
                Box::new(self.type_from_one(&ann[0])),
                Box::new(self.type_from_any(&ann[1..])),
            )
        }
    }

    pub fn get(&self, name: &String) -> Result<Type, TypeError> {
        match self.names.get(name) {
            Some(t)  => Ok(t.to_owned()),
            None     => Err(TypeError::InvalidTypeName)
        }
    }

    pub fn type_of(&self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Lit(lit) => match lit {
                Lit::Void      => Ok(Type::Void),
                Lit::I64(_)    => Ok(Type::I64),
                Lit::Bool(_)   => Ok(Type::Bool),
                Lit::String(_) => Ok(Type::String),
            },
            Expr::Name(name) => self.get(name),
            Expr::Apply { left, right } => {
                match self.type_of(left) {
                    Ok(Type::Func(input, output)) => {
                        match self.type_of(right) {
                            Ok(t) => {
                                if t == *input {
                                    Ok(*output)
                                } else {
                                    Err(TypeError::ArgOfWrongType {
                                        expr: expr.to_owned(),
                                        expected: *input,
                                        found:    t })
                                }
                            }
                            err => err
                        }
                    }
                    Ok(_) => Err(TypeError::ZeroArgFuncApplication { 
                                 expr: expr.to_owned() }),
                    err => err
                }
            }
        }
    }

    fn enter_func(&self, dfunc: &DFunc) -> Result<Self, TypeError> {
        let mut ctx = self.to_owned();
        for i in &dfunc.body {
            if let Instr::Declare(dfunc) = i {
                ctx.names.insert(
                    dfunc.name.to_string(),
                    ctx.type_from_any(&dfunc.ann),
                );
            }
        }
        // TODO: check params types
        // maybe defer this to the actual check, 
        // and only verify the number.
        // add param names to ctx, right now it's always 0 or 1.
        if dfunc.params.len() == 1 {
            match self.names.get(&dfunc.name) {
                Some(Type::Func(left, _)) => { 
                    ctx.names.insert(
                        dfunc.params[0].to_owned(),
                        *left.to_owned()
                    );
                    Ok(ctx) 
                },
                // None means the function isn't in the context,
                // which is impossible.
                _ => Err(TypeError::TypeAnnotationMismatch)
            }
        } else {
            Ok(ctx)
        }
    }

    fn check_one(ctx: &mut Self, instr: &Instr) -> Result<Type, TypeError> {
        match instr {
            Instr::Compute(expr) => ctx.type_of(expr),
            Instr::Bind { op: _, name, expr, ann } => {
                let etype = ctx.type_of(expr)?;
                if ctx.type_from_any(ann) == etype {
                    ctx.names.insert(name.to_owned(), etype.to_owned());
                    Ok(etype)
                } else {
                    Err(TypeError::TypeAnnotationMismatch)
                }
            }
            Instr::Declare(dfunc) => ctx.check(&dfunc),
            _ => unimplemented!(),
        }
    }

    pub fn check(&self, dfunc: &DFunc) -> Result<Type, TypeError> {
        let mut ctx = self.enter_func(dfunc)?;
        // Check that the other sequence of Instr's is well typed.
        // Check that the return value has the declared type.
        for i in &dfunc.body {
            Self::check_one(&mut ctx, i)?;
        };
        // FIXME: Redundant call
        match &dfunc.body.last() {
            Some(i) => { 
                let found = Self::check_one(&mut ctx, i)?;
                let expected = ctx.get(&dfunc.name)?;
                if found != expected {
                    Err(TypeError::InvalidReturnType)
                } else {
                    Ok(found)
                }
            },
            None => Err(TypeError::NoReturnValueFound)
        }
    }
}

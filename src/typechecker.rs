//!   This file describes the Woland Type System.
//!  
//!   In order to check if the program is correctly typed
//!   statically, we need a way to answer whether or not
//!   a give `Expr` is of type T. So we can make sure that
//!   function calls, Branches and Assigns get expressions
//!   of the correct type.
//!  
//!   To know if an Expr is of type T, we only need to check
//!   if that expression matches any of T's constructors!
//!   Hence, if
//!  
//!     type Division = Ok { num: I64 } | DivByZeroErr
//!  
//!   we know that `Ok 42` is of type Result, but `true`
//!   is not, as it's not within the list of constructors.
//!  
//!   For primitive types suck as I64, we check differently
//!   based on enum variants.
//!  
//!   An expression can either be a primitive literal; constructed
//!   through the underlying implementation, a variable reference,
//!   or a function application. These are the three cases where we
//!   need to check types.
//!  
//!   It follows that a type can be either a primitive type; hardcoded
//!   into the language implementation: I64, Bool, String, Void.
//!   (NOTE: some of these will be replaced by alternatives defined in
//!   the userland, once the type system is ready. Only I64 is sure to
//!   stay.)
//!   Moreover we get user-defined types through `type` decls, and lastly,
//!   an arrow type of the form `A -> B` representing a function. On top of this,
//!   we need to distinguish between Pure(A -> B) and Impure(A -> B) on the type
//!   level. For example:
//!   - Pure(I64) is a pure-function that evaluates to an I64.
//!   - Impure(I64) is a procedure that returns an I64.
//!   - Var(I64) is a variable of type I64.
//!   - Pure(I64 -> Bool) maybe a `even` function.
//!   - I64 -> Bool doesn't mean anything.
//!   TODO: Figure out in which direction this should be expanded to include
//!   types of the form `A1 -> A2 -> ... -> An`. The most obvious choice is
//!   to internally compile these into `A1 -> (A2 -> (... (...-> An)))` and
//!   provide appropriate syntax.

use std::collections::HashMap;

use crate::ast::*;
use crate::error::TypeError;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Primitive data types
    // handled by the implementation.
    Void,
    I64,
    Bool,
    String,
    // Type declared by the user
    New(HashMap<String, Constr>),
    // Type for single-argument, single-
    // result functions: T -> T
    Func(Box<Type>, Box<Type>),
}

pub type Constr = HashMap<String, Type>;

#[derive(Debug, Clone)]
pub struct Ctx {
    dtypes: HashMap<String, DType>,
    names: HashMap<String, Type>,
    vars: HashMap<String, Type>,
}

impl Ctx {
    pub fn new(prog: &AST) -> Result<Self, TypeError> {
        let mut ctx = Ctx {
            dtypes: HashMap::new(),
            names: HashMap::new(),
            vars: HashMap::new(),
        };
        for d in &prog.defs {
            match d {
                Def::Name(dfunc) => {
                    ctx.names
                        .insert(dfunc.name.to_string(), ctx.type_from_any(&dfunc.ann)?);
                }
                Def::Type(dtype) => {
                    ctx.dtypes.insert(dtype.name.to_string(), dtype.to_owned());
                }
            }
        }
        Ok(ctx)
    }

    /// Transforms a type annotation into a monotype.
    fn type_from_one(&self, ann: &str) -> Result<Type, TypeError> {
        match ann {
            "Void" => Ok(Type::Void),
            "I64" => Ok(Type::I64),
            "Bool" => Ok(Type::Bool),
            "String" => Ok(Type::String),
            _ => self.type_from_other(ann),
        }
    }

    /// Transforms a type annotation into a user-defined type.
    fn type_from_other(&self, ann: &str) -> Result<Type, TypeError> {
        match self.dtypes.get(ann) {
            Some(dconstrs) => {
                let mut constrs = HashMap::new();
                for dc in &dconstrs.body {
                    let mut c = HashMap::new();
                    if let Some(r) = &dc.record {
                        for (name, ann) in r {
                            c.insert(name.to_string(), self.type_from_any(ann)?);
                        }
                    }
                    constrs.insert(dc.name.to_string(), c);
                }
                Ok(Type::New(constrs))
            }
            None => Err(TypeError::InvalidTypeName {
                name: ann.to_owned(),
            }),
        }
    }

    /// Transforms any type annotation into a type.
    fn type_from_any(&self, ann: &[String]) -> Result<Type, TypeError> {
        if ann.len() == 1 {
            self.type_from_one(&ann[0])
        } else {
            Ok(Type::Func(
                Box::new(self.type_from_one(&ann[0])?),
                Box::new(self.type_from_any(&ann[1..])?),
            ))
        }
    }

    /// Queries the context for a name matching `name`.
    pub fn get(&self, name: &str) -> Result<Type, TypeError> {
        match self.names.get(name) {
            Some(t) => Ok(t.to_owned()),
            None => Err(TypeError::InvalidName {
                name: name.to_owned(),
            }),
        }
    }

    /// Computes the type of an arbitrary expression.
    pub fn type_of(&self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Void => Ok(Type::Void),
            Expr::I64(_) => Ok(Type::I64),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Str(_) => Ok(Type::String),
            Expr::Name(name) => self.get(name),
            Expr::Apply { left, right } => match self.type_of(left) {
                Ok(Type::Func(input, output)) => match self.type_of(right) {
                    Ok(t) => {
                        if t == *input {
                            Ok(*output)
                        } else {
                            Err(TypeError::ApplicationOnWrongType {
                                expr: *left.to_owned(),
                                expected: *input,
                                found: t,
                            })
                        }
                    }
                    err => err,
                },
                Ok(_) => Err(TypeError::ApplicationOfZeroArgFunc {
                    left: *left.to_owned(),
                    right: *right.to_owned(),
                }),
                err => err,
            },
            Expr::Func { .. } => unreachable!(),
            _ => {
                /* TDOO: Finish this! */
                Ok(Type::Void)
            }
        }
    }

    /// Checks that an `Instr` is correctly typed and if so, return its type.
    fn check_one(ctx: &mut Self, instr: &Instr) -> Result<Type, TypeError> {
        match instr {
            Instr::Compute(expr) => ctx.type_of(expr),
            Instr::Var {
                name,
                ann,
                op: _,
                expr,
            } => {
                let expected = ctx.type_from_any(ann)?;
                let found = ctx.type_of(expr)?;
                if expected == found {
                    ctx.vars.insert(name.to_owned(), expected);
                    // HACK: since the two types are equal, we can
                    // move them to different owners without cloning!
                    // Rust really is different.
                    Ok(found)
                } else {
                    Err(TypeError::AssignmentOfWrongType {
                        name: name.to_owned(),
                        expected,
                        found,
                    })
                }
            }
            Instr::Assign { name, op: _, expr } => {
                let expected = ctx.get(name)?;
                let found = ctx.type_of(expr)?;
                if expected == found {
                    Ok(found)
                } else {
                    Err(TypeError::AssignmentOfWrongType {
                        name: name.to_owned(),
                        expected,
                        found,
                    })
                }
            }
            Instr::Let(dname) => ctx.check(dname),
            Instr::Branch { paths } => {
                for b in paths {
                    let found = ctx.type_of(&b.0)?;
                    if found != Type::Bool {
                        return Err(TypeError::BranchConditionOfWrongType {
                            cond: b.0.to_owned(),
                            found,
                        });
                    }
                    for i in &b.1 {
                        Self::check_one(ctx, i)?;
                    }
                }
                Ok(Type::Void)
            }
            Instr::Loop { body } => {
                for i in body {
                    Self::check_one(ctx, i)?;
                }
                Ok(Type::Void)
            }
            // TODO: implement the rest!
            _ => Ok(Type::Void),
        }
    }

    /// Checks that a let-defintion is correctly typed.
    pub fn check(&self, dname: &DName) -> Result<Type, TypeError> {
        if let Expr::Func {
            ann,
            param,
            body,
            closure: _,
        } = dname.expr.to_owned()
        {
            if let Instr::Compute(Expr::Intrinsic { name, args: _ }) = body.first().unwrap() {
                if name.as_str() == "typeck_ignore" {
                    return Ok(Type::Void);
                }
            }
            // Enter's the function's context by absorbing all its contained defintions
            let mut ctx = self.to_owned();
            for i in &body {
                if let Instr::Let(dname) = i {
                    ctx.names
                        .insert(dname.name.to_string(), ctx.type_from_any(&dname.ann)?);
                }
                if let Instr::Var {
                    name,
                    ann,
                    op: _,
                    expr: _,
                } = i
                {
                    ctx.names.insert(name.to_string(), ctx.type_from_any(ann)?);
                }
            }
            ctx.names.insert(param, ctx.type_from_any(&dname.ann)?);
            // The parser doesn't allow functions with an empty list of Instr,
            // in the edge case, last exists and first is vec![].
            let (last, first) = body.split_last().unwrap();
            // Check that the other sequence of Instr's is well typed.
            for i in first {
                Self::check_one(&mut ctx, i)?;
            }
            // Check that the return value has the declared type.
            let ftype = ctx.get(&dname.name)?;
            let found = Self::check_one(&mut ctx, last)?;
            let expected = match &ftype {
                Type::Func(_, t) => *t.to_owned(),
                other => other.to_owned(),
            };
            if found == expected {
                Ok(ftype)
            } else {
                Err(TypeError::InvalidReturnType {
                    name: dname.name.to_owned(),
                    expected,
                    found,
                })
            }
        } else {
            Ok(Type::Void)
        }
    }
}

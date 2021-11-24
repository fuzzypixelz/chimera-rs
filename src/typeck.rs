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

use std::collections::HashMap;

use crate::ast::*;

struct Ctx {
    names: HashMap<String, Type>,
}

impl Ctx {
    fn new(func: &DFunc) -> Self {
        let mut ctx = Ctx {
            names: HashMap::new(),
        };
        for i in func.body {
            if let Instr::Decl(Decl::Func(dfunc)) = i {
                ctx.names.insert(dfunc.name, dfunc.sig);
            }
        }
        ctx
    }

    fn get(&self, name: &String) -> &Type {
        self.names
            .get(name)
            .unwrap_or_else(|| panic!("Woland: undefined reference to `{}`", name))
    }
}

impl Expr {
    fn is_type(&self, ty: &Type, ctx: &Ctx) -> bool {
        match self {
            Expr::Prim(prim) => match prim {
                Prim::Void => *ty == Type::Void,
                Prim::I64(_) => *ty == Type::I64,
                Prim::Bool(_) => *ty == Type::Bool,
                Prim::String(_) => *ty == Type::String,
            },
            Expr::Name(name) => *ty == *ctx.get(name),
            Expr::Apply { name, args } => {
                if let Type::Pure(_, result) = ctx.get(name) {
                    *result.as_ref() == *ty
                } else {
                    false
                }
            }
        }
    }
}

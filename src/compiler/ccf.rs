use crate::parser::ast::{Def, Expr, ItemKind, AST};
use thiserror::Error;

/// Core Chimera Form, a flatter AST.
#[derive(Debug)]
pub struct CCF {
    /// Sequence of bindings to expressions.
    pub defs: Vec<Def>,
    /// The main expression to evaluate, currently has no type contraints.
    /// Hence the main program is a function Void -> Void that evaluates this.
    pub main: Expr,
}

#[derive(Error, Debug)]
pub enum CCFError {
    #[error("no main expression was defined.")]
    NoMain,
}

impl TryFrom<AST> for CCF {
    type Error = CCFError;
    fn try_from(ast: AST) -> Result<Self, Self::Error> {
        // Search for the main expression.
        let main_index = ast
            .items
            .iter()
            .position(|i| {
                let ItemKind::Def(def) = &i.kind;
                def.name == "main"
            })
            .ok_or(CCFError::NoMain)?;

        let mut defs = ast
            .items
            .into_iter()
            .map(|i| {
                let ItemKind::Def(def) = i.kind;
                def
            })
            .collect::<Vec<_>>();
        let main = defs.remove(main_index).expr;

        Ok(CCF { defs, main })
    }
}

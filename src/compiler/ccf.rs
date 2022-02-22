use crate::check::cst::{Item, TypedExpr, CST};
use thiserror::Error;

/// Core Chimera Form, currently only extracts the main expression.
#[derive(Debug)]
pub struct CCF {
    /// Sequence of bindings to expressions.
    pub defs: Vec<Definition>,
    /// The main expression to evaluate, currently has no type contraints.
    /// Hence the main program is a function Void -> Void that evaluates this.
    pub main: TypedExpr,
}

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub texpr: TypedExpr,
}

#[derive(Error, Debug)]
pub enum CCFError {
    #[error("no main expression was defined.")]
    NoMain,
}

impl TryFrom<CST> for CCF {
    type Error = CCFError;
    fn try_from(cst: CST) -> Result<Self, Self::Error> {
        // Search for the main expression.
        let main_index = cst
            .items
            .iter()
            .position(|i| {
                let Item::Definition { name, .. } = &i;
                name == "main"
            })
            .ok_or(CCFError::NoMain)?;

        let mut defs = cst
            .items
            .into_iter()
            .map(|i| {
                // If the items is a Definition, retrieve it.
                let Item::Definition { name, texpr } = i;
                Definition { name, texpr }
            })
            .collect::<Vec<_>>();

        let main = defs.remove(main_index).texpr;

        Ok(CCF { defs, main })
    }
}

use super::fcf::{Bind, FlatExpr, Func, FCF};

/// Static single assignement, à la basic blocks.
#[derive(Debug)]
pub struct SSA {
    /// Top-level bindings with eliminated closures.
    pub binds: Vec<Bind>,
    /// Procedures in the form of basic blocks.
    pub procs: Vec<Proc>,
}

#[derive(Debug)]
pub struct Proc {
    pub name: String,
    pub param: usize,
    pub body: Vec<Block>,
}

#[derive(Debug)]
pub struct Block {
    pub label: String,
    pub params: Vec<String>,
    pub instrs: Vec<Instr>,
    pub transfer: Option<Transfer>,
}

#[derive(Debug)]
pub enum Instr {
    /// Define a function-local variable.
    Local(usize, FlatExpr),
}

#[derive(Debug)]
pub enum Transfer {
    /// Return a variable from the current function.
    Ret(usize),
}

/*
    let fortyTwo = |_| 42

    int fortyTwo(void) {
        entry():
        end():
            v1 = 42
            ret v1;
    }
*/

impl From<Func> for Proc {
    fn from(func: Func) -> Self {
        let Func { name, fexpr, .. } = func;
        let entry = Block {
            label: "entry".to_string(),
            params: vec![],
            instrs: vec![],
            transfer: None,
        };
        let end = Block {
            label: "end".to_string(),
            params: vec![],
            instrs: vec![Instr::Local(1, fexpr)],
            transfer: Some(Transfer::Ret(1)),
        };
        let body = vec![entry, end];
        let param = 0;
        Proc { name, param, body }
    }
}

impl From<FCF> for SSA {
    fn from(fcf: FCF) -> Self {
        let FCF { binds, funcs } = fcf;
        let procs = funcs // Use the above From trait.
            .into_iter()
            .map(Proc::from)
            .collect::<Vec<_>>();
        SSA { binds, procs }
    }
}

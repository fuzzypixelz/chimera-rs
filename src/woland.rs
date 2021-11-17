/*
    This file is the Woland Tree-Walk Interpreter.

    Following the parser's generation of the AST, this
    module executes the program described by the tree
    directly, i.e no IR is used.

    Hence, it contains the following facilities for
    simulating a runtime:

    A. Environments.

      1. A notion of proc/pure function local environment,
      consisting of a HashMap between binding names and Expr's.
      2. A global HashMap mapping each proc/pure function's
      name to a local environment.

    B. Expression evaluation.

      1. A means of evaluating any Expr (recursively), this would
      require access to the proc/pure's local environment.
      2. Currently all evaluation produces only Woland's primitive
      types. In the future there will be support for compound data
      types as well as abstract data types. The current primitives
      are: Bool, I64, U64, F64, String.

    C. Proc execution.
      1. As it stands, a proc is a sequence of Instr's:
      enum Instr {
          Expr(Expr), // Produces a value (B) and may have side-
                      // effects, useful for returns and calls.
          Bind(Bind), // Changes the proc's local env, useful
                      // for giving names to Expr's.
      }
      2. A procedure's last instruction will always be an Expr,
      this is what we evaluate its Calls to.

    D. Imports
      1. importing a file means parsing it into an AST, and including
      only the `decls` presents in its exports section, otheriwse
      we include everything by "merging" the two `decls` fields.
*/
use std::collections::HashMap;
use std::io;

use crate::ast::*;

#[derive(Clone)]
pub struct Env {
    names: HashMap<String, Expr>,
    loops: Vec<Loop>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            loops: vec![],
        }
    }
}

impl Expr {
    fn eval<'a>(&'a self, env: &'a Env, ast: &AST) -> Prim {
        match self {
            Expr::Prim(prim) => prim.to_owned(),
            Expr::Name(id) => env
                .names
                .get(id)
                .unwrap_or_else(|| panic!("Woland: undefined reference to {} identifier.", id))
                .eval(env, ast),
            Expr::Call(Call { func_name, args }) => {
                if !ast.decls.contains_key(func_name) {
                    // panic!("Woland: undefined reference to {} procedure.", proc_name);
                };

                match func_name.as_str() {
                    // TODO: This is a big hack to simulate std lib functions.
                    "dump" => {
                        if args.len() == 0 {
                            panic!("Woland: dump takes at least one argument.")
                        }
                        println!(
                            "{:?}",
                            args.iter().map(|a| a.eval(env, ast)).collect::<Vec<Prim>>()
                        );
                        Prim::I64(0)
                    }
                    "read" => {
                        let mut buffer = String::new();
                        io::stdin()
                            .read_line(&mut buffer)
                            .expect("Woland: error reading from stdin. You are on your own.");
                        Prim::String(buffer)
                    }
                    _ => {
                        let mut env: Env = Env::new();
                        let Decl::Func(func) = &ast
                            .decls
                            .get(func_name)
                            .unwrap_or_else(|| panic!("Woland: {} is not a function.", func_name));
                        if func.kind.params.len() == 0 {
                            panic!("Woland: no type inference yet!")
                        }
                        if func.kind.params.len() != args.len() {
                            panic!(
                                "Woland: function {} expects {} arguments, not {}.",
                                func_name,
                                func.kind.params.len(),
                                args.len()
                            )
                        }
                        for ((k, _), v) in func.kind.params.iter().zip(args.iter()) {
                            env.names.insert(k.to_owned(), v.to_owned());
                        }
                        func.run(&mut env, ast)
                    }
                }
            }
        }
    }
}

impl Instr {
    fn execute(&self, env: &mut Env, ast: &AST) {
        match self {
            Instr::Bind(Bind { id, ty: _, expr }) => {
                env.names.insert(id.to_owned(), expr.to_owned());
            }
            Instr::Expr(expr) => {
                // The evaluated expression may or may not have
                // any side-effects. Beware!
                expr.eval(env, ast);
            }
            Instr::Cond(Cond { cond, fst, snd }) => match cond.eval(env, ast) {
                Prim::Bool(b) => {
                    if b {
                        fst.iter().for_each(|i| i.execute(env, ast))
                    } else {
                        snd.iter().for_each(|i| i.execute(env, ast))
                    }
                }
                _ => panic!("Woland: TypeError: expected Bool."),
            },
            Instr::Loop(loop_) => {
                env.loops.push(loop_.to_owned());
                while env.loops.len() != 0 {
                    env.to_owned()
                        .loops
                        .last()
                        .unwrap()
                        .body
                        .iter()
                        .for_each(|i| i.execute(env, ast))
                }
            }
            Instr::Keyword(keyword) => match keyword {
                Keyword::Break => {
                    env.loops
                        .pop()
                        .unwrap_or_else(|| panic!("Woland: can only break out of a loop."));
                }
            },
        }
    }
}

impl Func {
    pub fn run<'a>(&'a self, env: &'a mut Env, ast: &AST) -> Prim {
        if self.body.len() == 0 {
            panic!("Woland: empty function body.");
        }
        for instr in &self.body[..self.body.len() - 1] {
            instr.execute(env, ast);
        }
        match self.body.last().unwrap() {
            Instr::Expr(ret) => ret.eval(env, ast),
            _ => panic!("Woland: expected expression at function's end."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Prim::*;

    #[test]
    fn eval_prim_I64() {
        assert_eq!(
            Expr::Prim(I64(0)).eval(
                &Env::new(),
                &AST {
                    decls: HashMap::new()
                }
            ),
            I64(0)
        );
    }
}

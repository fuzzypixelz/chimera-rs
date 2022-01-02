mod ast;
mod code;
mod compiler;
mod error;
mod lexer;
mod typechecker;
mod value;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

use anyhow::{Context, Error, Result};
use ast::{Def, Instr, AST};
use code::{Code, Cont, Env};
use error::fmt_parse_error;
use lexer::Lexer;
use std::{cell::RefCell, collections::HashMap, env, fs, rc::Rc};
use typechecker::Lexicon;

/*
    import core/io (println)
    export ()

    let main ~ do
      -- The answer to life,
      -- the universe and everything.
      println 42
    end
*/

fn read_program(filename: String) -> Result<AST> {
    let source = fs::read_to_string(&filename).with_context(|| {
        format!(
            "error reading source file `{}`, you are on your own.",
            filename
        )
    })?;
    let lexer = Lexer::new(&source);
    let result = grammar::ASTParser::new().parse(&source, &mut HashMap::new(), lexer);
    match result {
        Ok(program) => {
            // TODO: Is it necessary to create a new Lexicon for each file we import?
            Lexicon::default().check(&program.defs).with_context(|| {
                format!("encountered a type error in source file `{}`", filename)
            })?;
            Ok(program)
        }
        Err(error) => Err(Error::msg(fmt_parse_error(&source, error))),
    }
}

fn main() -> Result<()> {
    let filename = env::args()
        .nth(1)
        .with_context(|| "no src file was specified")?;
    let mut program = read_program(filename.clone())
        .with_context(|| format!("failed to parse source file `{}`", filename))?;
    for mut filename in program.module.imports {
        filename.push_str(".wo"); // The parser doesn't add the extension.
        let mut import = read_program(filename.to_string())
            .with_context(|| format!("failed to parse imported source file `{}`", filename))?;
        import.defs.extend(program.defs.into_iter());
        program = import;
    }
    let env = Rc::new(RefCell::new(Env::default()));
    let cont = Rc::new(RefCell::new(Cont::default()));
    for def in program.defs {
        match def {
            Def::Name(dname) => {
                let instr = Instr::Let(dname);
                instr.compile().execute(env.clone(), cont.clone());
            }
        }
    }
    Ok(())
}

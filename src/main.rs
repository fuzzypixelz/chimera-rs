#[macro_use]
extern crate lalrpop_util;

use std::cell::RefCell;
use std::rc::Rc;
use std::{env, fs};

use anyhow::{Context, Result};

use parser::parse;

use crate::code::{Code, Env};

// use crate::typechecker::Lexicon;

mod ast;
mod attribute;
mod code;
mod compiler;
mod error;
mod lexer;
mod parser;
mod typechecker;
mod value;

lalrpop_mod!(#[allow(clippy::all)] pub grammar);

/*
    use core::io::println

    let main ~ do
        -- The answer to life,
        -- the universe and everything.
        println 42
    end
*/

fn main() -> Result<()> {
    let mut program = Vec::new();
    for filename in env::args().skip(1) {
        let source = fs::read_to_string(&filename).with_context(|| {
            format!(
                "error reading source file `{}`, you are on your own.",
                filename
            )
        })?;
        let items = parse(&source)
            .with_context(|| format!("error while parsing source file `{}`", filename))?;

        // let lexicon = Lexicon::default();
        // for item in &items {
        //     lexicon
        //         .check(item)
        //         .with_context(|| format!("encountered a type error in source file `{}`", filename))?;
        // }

        program.extend(items);
    }

    let env = Rc::new(RefCell::new(Env::default()));
    for item in program {
        item.compile().execute(env.clone());
    }
    Ok(())
}

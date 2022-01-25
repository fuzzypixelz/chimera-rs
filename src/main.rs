#[macro_use]
extern crate lalrpop_util;

use std::{env, fs};
use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};

use parser::parse;

use crate::code::{Code, Cont, Env};

// use crate::typechecker::Lexicon;

mod ast;
mod code;
mod error;
mod lexer;
mod parser;
mod typechecker;
mod value;
mod compiler;
mod attribute;

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
    let filename = env::args()
        .nth(1)
        .with_context(|| "no source file was specified")?;

    let source = fs::read_to_string(&filename).with_context(|| {
        format!(
            "error reading source file `{}`, you are on your own.",
            filename
        )
    })?;

    let program = parse(&source)
        .with_context(|| format!("error while parsing source file `{}`", filename))?;

    // let lexicon = Lexicon::default();
    // for item in &program {
    //     lexicon
    //         .check(item)
    //         .with_context(|| format!("encountered a type error in source file `{}`", filename))?;
    // }

    let env = Rc::new(RefCell::new(Env::default()));
    let cont = Rc::new(RefCell::new(Cont::default()));
    for item in program {
        item.compile()
            .execute(env.clone(), cont.clone());
    }
    Ok(())
}

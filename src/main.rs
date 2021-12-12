mod lexer;
mod ast;
mod typechecker;
mod error;
mod interpreter;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

use std::env;
use std::fs;

use ast::*;
use lexer::Lexer;
use typechecker::Ctx;
use interpreter::Env;
use anyhow::Result;

/*
    import core/io
    export ()

    let main: Void ~
      // The answer to life,
      // the universe and everything.
      dmp 42
    end
*/
fn main() -> Result<()> {
    let filename = env::args()
        .nth(1)
        .expect("Woland: no source file was specified.");
    let source = fs::read_to_string(filename.to_owned())
        .expect("Woland: error reading source file. You are on your own.");
    let lexer = Lexer::new(&source);
    let program = grammar::ASTParser::new()
        .parse(&source, lexer)
        .unwrap();
    // print!("Woland is executing:\n{}", program);
    let ctx = Ctx::new(&program)?;
    for d in &program.defs {
        if let Def::Name(dname) = d {
            ctx.check(dname)?;
        }
    }
    let env = Env::new(&program);
    let entry = env
        .get(&String::from("main"))
        .to_owned();
        // .expect("Woland: the main function was never declared.");
    
    let main = Expr::Apply { 
        left: Box::new(entry), 
        // TODO: put all of argv into here!
        right: Box::new(Expr::Str(filename))
    };
    main.eval(&env);
    Ok(())
}

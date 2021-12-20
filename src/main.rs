mod ast;
// mod error;
mod lexer;
// mod typechecker;
mod interpreter;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

use anyhow::Result;
use ast::Def;
use interpreter::{Cont, Env};
use lexer::Lexer;
use rustc_hash::FxHashMap;
// use typechecker::Ctx;

/*
    import core/io
    export ()

    let main: Void ~ do
      # The answer to life,
      # the universe and everything.
      dump 42
    end
*/
fn main() -> Result<()> {
    let filename = env::args()
        .nth(1)
        .expect("Woland: no source file was specified.");
    let source = fs::read_to_string(filename)
        .expect("Woland: error reading source file. You are on your own.");
    let lexer = Lexer::new(&source);
    let program = grammar::ASTParser::new()
        .parse(&source, &mut FxHashMap::default(), lexer)
        .unwrap();
    // print!("Woland is executing:\n{:#?}", program);
    // let ctx = Ctx::new(&program)?;
    // for d in &program.defs {
    //     if let Def::Name(dname) = d {
    //         ctx.check(dname)?;
    //     }
    // }
    let env = Rc::new(RefCell::new(Env::new(&program)));
    let cont = Rc::new(RefCell::new(Cont::default()));
    // let entry = env
    //     .borrow()
    //     .names
    //     .get(&String::from("main"))
    //     .expect("Woland: the main function was never declared.")
    //     .clone();

    // let main = Expr::Apply {
    //     left: Box::new(entry),
    //     // TODO: put all of argv into here!
    //     right: Box::new(Expr::Str(filename)),
    // };
    // entry.eval(env, cont);
    for def in program.defs {
        if let Def::Name(dname) = def {
            let rhs = dname.expr.eval(env.clone(), cont.clone());
            env.borrow_mut().names.insert(dname.name, rhs);
        }
    }
    Ok(())
}

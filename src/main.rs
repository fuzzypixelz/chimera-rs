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
// use typechecker::Ctx;
use ast::AST;

/*
    import core/io
    export ()

    let main: Void ~ do
      # The answer to life,
      # the universe and everything.
      dump 42
    end
*/
fn read_program(filename: String) -> AST {
    let source = fs::read_to_string(filename)
        .expect("Woland: error reading source file. You are on your own.");
    let lexer = Lexer::new(&source);
    grammar::ASTParser::new().parse(&source, lexer).unwrap()
}

fn main() -> Result<()> {
    let filename = env::args()
        .nth(1)
        .expect("Woland: no source file was specified.");
    let mut program = read_program(filename);
    // print!("Woland is executing:\n{:#?}", program);
    // let ctx = Ctx::new(&program)?;
    // for d in &program.defs {
    //     if let Def::Name(dname) = d {
    //         ctx.check(dname)?;
    //     }
    // }
    for mut filename in program.module.imports {
        filename.push_str(".wo"); // The parser doesn't add the extension.
        let mut import = read_program(filename.to_string());
        import.defs.extend(program.defs.into_iter());
        program = import;
    }
    let env = Rc::new(RefCell::new(Env::new(&program.defs)));
    let cont = Rc::new(RefCell::new(Cont::default()));
    for def in program.defs {
        match def {
            Def::Name(dname) => {
                let rhs = dname.expr.eval(env.clone(), cont.clone());
                env.borrow_mut().names.insert(dname.name, rhs);
            }
        }
    }
    Ok(())
}

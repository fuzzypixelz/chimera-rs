mod ast;
mod parser;
mod typeck;
mod woland;

use std::env;
use std::fs;

use crate::ast::Decl;
use crate::parser::ast;
use crate::woland::Env;
use nom::Finish;

/*
    import core/io
    export ()

    let main: Void ~
      // The answer to life,
      // the universe and everything.
      dmp 42
    end
*/

fn main() {
    let filename = env::args()
        .nth(1)
        .expect("Woland: no source file was specified.");
    let source = fs::read_to_string(filename)
        .expect("Woland: error reading source file. You are on your own.");
    let (_, program) = ast(&source).finish().unwrap();
    // println!("Executing:\n{:?}\n", program);
    let Decl::Func(entry) = &program
        .decls
        .get(&String::from("main"))
        .expect("Woland: the main function was never declared.");
    entry.run(&mut Env::new(), &program);
}

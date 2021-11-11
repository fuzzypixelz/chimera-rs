mod ast;
mod parser;
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

    proc main -> Void is
      // The answer to life,
      // the universe and everything.
      dump(42)
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
    let Decl::Proc(entry) = &program
        .decls
        .get(&String::from("main"))
        .expect("Woland: the main procedure was never declared.");
    entry.run(&mut Env::new(), &program);
}

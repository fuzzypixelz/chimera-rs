use std::env;
use std::fs;

use common::Decl;
use parser::run;
use typechecker::Ctx;
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
    let source = fs::read_to_string(filename)
        .expect("Woland: error reading source file. You are on your own.");
    let program = run(&source);
    let ctx = Ctx::new(&program);
    for d in &program.decls {
        if let Decl::Func(dfunc) = d {
            ctx.check(dfunc)?;
        }
    }
    // println!("Executing:\n{:?}\n", program);
    // let Decl::Func(entry) = &program
    //     .decls
    //     .get(&String::from("main"))
    //     .expect("Woland: the main function was never declared.");
    // entry.run(&mut Env::new(), &program);
    Ok(())
}

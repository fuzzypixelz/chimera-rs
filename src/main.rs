#![deny(elided_lifetimes_in_paths)]

extern crate lalrpop_util;

mod check;
mod codegen;
mod compiler;
mod error;
mod mlir;
mod parser;
// mod interpreter;

use anyhow::{Context, Error, Result};
use check::cst::CST;
use clap::{command, Arg};
use compiler::{ccf::CCF, fcf::FCF};
use parser::{ast::AST, Parser};
use std::fs;

fn main() -> Result<()> {
    let matches = command!()
        .arg(
            Arg::new("ir")
                .short('e')
                .long("emit")
                .takes_value(true)
                .help("Print out intermediate representations")
                .possible_values(["ast", "cst", "ccf", "fcf"])
                .long_help(
                    "Print out intermediate representations:\n\
                     - AST: (Abstract Syntax Tree) the result of running the parser.\n\
                     - CST: (Concrete Syntax Tree) fully typed version of the AST.\n\
                     - CCF: (Core Chimera Form) sequence of definition and a main expression.\n\
                     - FCF: (Flat Chimera Form) all lambdas are lifted into top level functions.",
                ),
        )
        .arg(
            Arg::new("FILES")
                .long_help("list of input files to be compiled")
                .multiple_values(true),
        )
        .get_matches();

    if let Some(filnames) = matches.values_of("FILES") {
        let mut ast = AST { items: Vec::new() };
        for filename in filnames {
            let source = fs::read_to_string(&filename)
                .with_context(|| format!("error reading source file `{}`.", filename))?;
            let items = Parser::new(&source)
                .run()
                .with_context(|| format!("error while parsing source file `{}`", filename))?
                .items;

            ast.items.extend(items);
        }
        if let Some("ast") = matches.value_of("ir") {
            eprintln!("{ast:#?}")
        }
        let cst = CST::try_from(ast)?;
        if let Some("cst") = matches.value_of("ir") {
            eprintln!("{cst:#?}")
        }
        let ccf = CCF::try_from(cst)?;
        if let Some("ccf") = matches.value_of("ir") {
            eprintln!("{ccf:#?}")
        }
        let fcf = FCF::from(ccf);
        if let Some("fcf") = matches.value_of("ir") {
            eprintln!("{fcf:#?}")
        }
        Ok(())
    } else {
        Err(Error::msg("no source files were supplied."))
    }
}

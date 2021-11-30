mod lexer;

use common::*;
use crate::lexer::Lexer;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

pub fn run(input: &str) -> AST {
    let lexer = Lexer::new(input);
    grammar::ASTParser::new()
        .parse(input, lexer)
        .unwrap()
}

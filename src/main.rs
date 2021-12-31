mod ast;
// mod error;
mod lexer;
// mod typechecker;
mod code;
mod compiler;
mod value;

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use anyhow::{Context, Error, Result};
use code::{Code, Cont, Env};
use lalrpop_util::ParseError;
use lexer::Lexer;
use std::{cell::RefCell, collections::HashMap, env, fs, rc::Rc};
// use typechecker::Ctx;
use ast::{Def, Instr, AST};

/*
    import core/io (dump)
    export ()

    let main: Void ~ do
      # The answer to life,
      # the universe and everything.
      dump 42
    end
*/

fn parse_error(source: &str, label: &str, slice_label: &str, range: (usize, usize)) -> Error {
    let snippet = Snippet {
        title: Some(Annotation {
            label: Some(label),
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices: vec![Slice {
            source,
            line_start: 0,
            origin: None,
            fold: true,
            annotations: vec![SourceAnnotation {
                label: slice_label,
                annotation_type: AnnotationType::Error,
                range,
            }],
        }],
        opt: FormatOptions {
            color: true,
            ..Default::default()
        },
    };
    Error::msg(DisplayList::from(snippet).to_string())
}

fn read_program(filename: String) -> Result<AST> {
    let source = fs::read_to_string(&filename).with_context(|| {
        format!(
            "error reading source file `{}`, you are on your own.",
            filename
        )
    })?;
    let lexer = Lexer::new(&source);
    let result = grammar::ASTParser::new().parse(&source, &mut HashMap::new(), lexer);
    match result {
        Ok(program) => Ok(program),
        Err(error) => match error {
            ParseError::InvalidToken { location } => Err(parse_error(
                &source,
                "invalid token",
                "invalid token",
                (location, location),
            )),
            ParseError::UnrecognizedEOF { location, expected } => {
                let label = format!("unrecognized EOF, expected {}", expected.join(", "));
                Err(parse_error(
                    &source,
                    label.as_str(),
                    "unrecognized EOF",
                    (location, location),
                ))
            }
            ParseError::ExtraToken { token } => {
                let label = format!("unexpected additonal token {}", token.1);
                let slice_label = format!("found extra `{}`", token.1);
                Err(parse_error(
                    &source,
                    label.as_str(),
                    slice_label.as_str(),
                    (token.0, token.2),
                ))
            }
            ParseError::UnrecognizedToken { token, expected } => {
                let label = format!(
                    "unrecognized token {}, expected {}",
                    token.1,
                    expected.join(", ")
                );
                let slice_label = format!("found `{}`", token.1);
                Err(parse_error(
                    &source,
                    label.as_str(),
                    slice_label.as_str(),
                    (token.0, token.2),
                ))
            }
            ParseError::User { .. } => unreachable!(),
        },
    }
}

fn main() -> Result<()> {
    let filename = env::args()
        .nth(1)
        .with_context(|| "no source file was specified")?;
    let mut program = read_program(filename.clone())
        .with_context(|| format!("failed to parse source file `{}`", filename))?;
    for mut filename in program.module.imports {
        filename.push_str(".wo"); // The parser doesn't add the extension.
        let mut import = read_program(filename.to_string())
            .with_context(|| format!("failed to parse imported source file `{}`", filename))?;
        import.defs.extend(program.defs.into_iter());
        program = import;
    }
    let env = Rc::new(RefCell::new(Env::default()));
    let cont = Rc::new(RefCell::new(Cont::default()));
    for def in program.defs {
        match def {
            Def::Name(dname) => {
                let instr = Instr::Let(dname);
                instr.compile().execute(env.clone(), cont.clone());
            }
        }
    }
    Ok(())
}

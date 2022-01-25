use std::collections::HashMap;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use anyhow::{Error, Result};
use lalrpop_util::ParseError;

use crate::{ast::Item, lexer::Tok};
use crate::error::LexicalError;
use crate::lexer::Lexer;

pub fn parse(source: &str) -> Result<Vec<Item>> {
    let lexer = Lexer::new(source);
    let result = crate::grammar::ProgramParser::new().parse(source, &mut HashMap::new(), lexer);
    match result {
        Ok(program) => Ok(program),
        Err(error) => Err(Error::msg(fmt_parse_error(source, error))),
    }
}

/// Takes information extracted from a `TypeError` and the relevant source code
/// to produce a pretty printed annotated-snippet. This is meant to be wrapped
/// in `anyhow::Error::msg` for use in `main`.
fn fmt_parse_error(source: &str, error: ParseError<usize, Tok<'_>, LexicalError>) -> String {
    // NOTE: One cannot impl Display for ParseError since it's defined
    // in an external crate, and lalrpop_util implements it anyway.
    // The output, however, leaves much to be desired. Hence why you
    // see an `fmt_parse_error` function here. Not very idiomatic of me.
    match error {
        ParseError::InvalidToken { location } => ann_parse_error(
            source,
            "invalid token",
            "invalid token",
            (location, location),
        ),
        ParseError::UnrecognizedEOF { location, expected } => {
            let label = format!("unrecognized EOF, expected {}", expected.join(", "));
            ann_parse_error(
                source,
                label.as_str(),
                "unrecognized EOF",
                (location, location),
            )
        }
        ParseError::ExtraToken { token } => {
            let label = format!("unexpected additonal token {}", token.1);
            let slice_label = format!("found extra `{}`", token.1);
            ann_parse_error(
                source,
                label.as_str(),
                slice_label.as_str(),
                (token.0, token.2),
            )
        }
        ParseError::UnrecognizedToken { token, expected } => {
            let label = format!(
                "unrecognized token {}, expected {}",
                token.1,
                expected.join(", ")
            );
            let slice_label = format!("found `{}`", token.1);
            ann_parse_error(
                source,
                label.as_str(),
                slice_label.as_str(),
                (token.0, token.2),
            )
        }
        ParseError::User { .. } => unreachable!(),
    }
}

fn ann_parse_error(source: &str, label: &str, slice_label: &str, range: (usize, usize)) -> String {
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
    DisplayList::from(snippet).to_string()
}

#[cfg(test)]
mod tests {
    use ::polytype::*;

    use crate::ast::*;

    use super::*;

    #[test]
    fn empty_program() {
        let source = r"";
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(result, Ok(vec![]))
    }

    #[test]
    fn definition_int() {
        let source = "let answer = 42\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "answer".to_string(),
                    ann: None,
                    expr: Expr::Int(42),
                },
            }])
        )
    }

    #[test]
    fn definition_bool() {
        let source = "let truth = true\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "truth".to_string(),
                    ann: None,
                    expr: Expr::Bool(true),
                },
            }])
        )
    }

    #[test]
    fn definition_char() {
        let source = "let most_iconic_lang = 'C'\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "most_iconic_lang".to_string(),
                    ann: None,
                    expr: Expr::Char('C'),
                },
            }])
        )
    }

    #[test]
    fn definition_str() {
        let source = "let hello = \"Hello, World!\"\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "hello".to_string(),
                    ann: None,
                    expr: Expr::Str("Hello, World!".to_string()),
                },
            }])
        )
    }

    #[test]
    fn definition_ident() {
        let source = "let hello = hi\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "hello".to_string(),
                    ann: None,
                    expr: Expr::Name("hi".to_string()),
                },
            }])
        )
    }

    #[test]
    fn definition_branch() {
        let source = "let one = if true then 1 end\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::Definition {
                    name: "hello".to_string(),
                    ann: None,
                    expr: Expr::Branch {
                        paths: vec![(Expr::Bool(true), vec![Stmt::Expr(Expr::Int(1))])]
                    },
                },
            }])
        )
    }

    #[test]
    fn definition_with_attr() {
        let source = "@[intrinsic(())]\nlet name_with_attr: Void = ()\n";
        dbg!(source.len());
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![Attr {
                    name: "intrinsic".to_string(),
                    args: vec![Expr::Void],
                }],
                kind: ItemKind::Definition {
                    name: "name_with_attr".to_string(),
                    ann: Some(ptp!(Void)),
                    expr: Expr::Void,
                },
            }])
        )
    }

    #[test]
    fn datatype_empty_constructor() {
        let source = r"data Direction
            Left {},
            Right {},
            Up {},
            Down {},
        end
        ";
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::DataType {
                    schema: ptp!(Direction),
                    variants: vec![
                        ("Left".to_string(), vec![]),
                        ("Right".to_string(), vec![]),
                        ("Up".to_string(), vec![]),
                        ("Down".to_string(), vec![]),
                    ],
                },
            }])
        )
    }

    #[test]
    fn datatype_one_constructor() {
        let source = r"data Person
            Person {
                name: Str,
                age: Int,
                job: Job
            },
        end
        ";
        let lexer = Lexer::new(&source);
        let result =
            crate::grammar::ProgramParser::new().parse(&source, &mut HashMap::new(), lexer);
        assert_eq!(
            result,
            Ok(vec![Item {
                attrs: vec![],
                kind: ItemKind::DataType {
                    schema: ptp!(Person),
                    variants: vec![(
                        "Person".to_string(),
                        vec![
                            ("name".to_string(), ptp!(Str)),
                            ("age".to_string(), ptp!(Int)),
                            ("job".to_string(), ptp!(Job)),
                        ]
                    )],
                },
            }])
        )
    }
}

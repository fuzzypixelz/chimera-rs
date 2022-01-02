use crate::lexer::Tok;
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use lalrpop_util::ParseError;
use polytype::UnificationError;
use thiserror::Error;

#[derive(Error, Copy, Clone, Debug)]
pub enum LexicalError {
    #[error("Invalid syntax")]
    InvalidSyntax,
}

/// Takes information extracted from a `TypeError` and the relevant source code
/// to produce a pretty printed annotated-snippet. This is meant to be wrapped
/// in `anyhow::Error::msg` for use in `main`.
pub fn fmt_parse_error(source: &str, error: ParseError<usize, Tok<'_>, LexicalError>) -> String {
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

#[derive(Error, Debug)]
pub enum TypeError {
    // FIXME: do proper error reporting, this would require:
    //  1. translating this error to a human-readable version
    //  2. recording information about where in the source code
    //     the expression shows and use annotate-snippets.
    #[error("unification error")]
    UnificationError(#[from] UnificationError),
    #[error("the name `{0}` is not in scope")]
    ScopeError(String),
}

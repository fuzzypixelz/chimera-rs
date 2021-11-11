/*
    This is a prototyping parser for Woland.

    I'm not yet sure what the syntax will look like
    in the end, so you'll find no grammar in here.
*/

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, char, i64, multispace0, newline, space0, space1},
    combinator::{opt, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};

use crate::ast::*;

fn literal(input: &str) -> IResult<&str, Expr> {
    let (input, int) = i64(input)?;
    use Prim::*;
    Ok((input, Expr::Prim(I64(int))))
}

fn call(input: &str) -> IResult<&str, Expr> {
    let (input, proc) = alpha1(input)?;
    let (input, _) = char('(')(input)?;
    let (input, args) = opt(many1(expr))(input)?;
    let (input, _) = char(')')(input)?;

    Ok((
        input,
        Expr::Call(Call {
            proc_name: proc.to_string(),
            args,
        }),
    ))
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    terminated(inner, multispace0)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn identifier_typed(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(ws(identifier), ws(char(':')), identifier)(input)
}

fn bind(input: &str) -> IResult<&str, Instr> {
    let (input, _) = tag("let")(input)?;
    let (input, _) = space1(input)?;
    let (input, (id, ty)) = identifier_typed(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = alt((tag("~"), tag("=")))(input)?;
    let (input, _) = space1(input)?;
    let (input, expr) = expr(input)?;
    let (input, _) = newline(input)?;
    Ok((
        input,
        Instr::Bind(Bind {
            id: id.to_string(),
            ty: ty.to_string(),
            expr,
        }),
    ))
}

fn expr(input: &str) -> IResult<&str, Expr> {
    let id_expr = |i| -> IResult<&str, Expr> {
        let (i, id) = identifier(i)?;
        Ok((i, Expr::Id(id.to_string())))
    };

    let (input, _) = space0(input)?; // HACK: this doesn't properly enforce identation.
    let (input, expr) = alt((literal, call, id_expr))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, expr))
}

fn expr_instr(input: &str) -> IResult<&str, Instr> {
    let (input, expr) = expr(input)?;
    let (input, _) = newline(input)?;
    Ok((input, Instr::Expr(expr)))
}

fn proc_kind(input: &str) -> IResult<&str, (Option<Vec<(String, String)>>, String)> {
    let (input, arg_kind) = opt(delimited(
        ws(char('(')),
        separated_list1(ws(tag(",")), ws(identifier_typed)),
        ws(char(')')),
    ))(input)?;
    let (input, _) = ws(tag("->"))(input)?;
    let (input, ret_kind) = ws(identifier)(input)?;
    Ok((
        input,
        (
            match arg_kind {
                Some(args) => Some(
                    args.iter()
                        .map(|(s, t)| (s.to_string(), t.to_string()))
                        .collect(),
                ),
                None => None,
            },
            ret_kind.to_string(),
        ),
    ))
}

pub fn proc(input: &str) -> IResult<&str, (String, Decl)> {
    let (input, _) = ws(tag("proc"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, kind) = ws(proc_kind)(input)?;
    let (input, _) = ws(tag("is"))(input)?;
    let (input, body_input) = take_until("end")(input)?;
    let (_, body) = many0(alt((ws(bind), ws(expr_instr))))(body_input)?;
    let (input, _) = ws(tag("end"))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, (name.to_string(), Decl::Proc(Proc { kind, body }))))
}

pub fn ast(input: &str) -> IResult<&str, AST> {
    let (input, decls) = many1(proc)(input)?;
    Ok((
        input,
        AST {
            decls: decls.into_iter().collect(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Prim::*;

    #[test]
    fn basic_literal() {
        assert_eq!(literal("42"), Ok(("", Expr::Prim(I64(42)))));
    }

    #[test]
    fn basic_proc_call() {
        assert_eq!(
            call("dump(666 my_favourite_number)"),
            Ok((
                "",
                Expr::Call(Call {
                    proc_name: "dump".to_string(),
                    args: Some(vec![
                        Expr::Prim(I64(666)),
                        Expr::Id("my_favourite_number".to_string())
                    ])
                })
            ))
        );
    }

    #[test]
    fn basic_bind() {
        assert_eq!(
            bind("let number: I8 ~ 69\n"),
            Ok((
                "",
                Instr::Bind(Bind {
                    id: "number".to_string(),
                    ty: "I8".to_string(),
                    expr: Expr::Prim(I64(69))
                })
            ))
        )
    }

    #[test]
    fn id_with_type() {
        assert_eq!(identifier_typed("x: I64"), Ok(("", ("x", "I64"))));
    }

    #[test]
    fn proc_kind_two_args() {
        assert_eq!(
            proc_kind("(x: I64, y: I64) -> I64"),
            Ok((
                "",
                (
                    Some(vec![
                        ("x".to_string(), "I64".to_string()),
                        ("y".to_string(), "I64".to_string())
                    ]),
                    "I64".to_string()
                )
            ))
        )
    }

    #[test]
    fn basic_proc() {
        assert_eq!(
            proc("proc main -> I32 is\n    -1\n end"),
            Ok((
                "",
                (
                    "main".to_string(),
                    Decl::Proc(Proc {
                        kind: (None, "I32".to_string()),
                        body: vec![Instr::Expr(Expr::Prim(I64(-1)))],
                    })
                )
            ))
        )
    }

    #[test]
    fn ast_two_procs() {
        assert_eq!(
            ast("proc whatever -> I8 is\n   0\n end\n\nproc main -> I64 is\n   -1\n end"),
            Ok((
                "",
                AST {
                    decls: vec![
                        (
                            "whatever".to_string(),
                            Decl::Proc(Proc {
                                kind: (None, "I8".to_string()),
                                body: vec![Instr::Expr(Expr::Prim(I64(0)))],
                            })
                        ),
                        (
                            "main".to_string(),
                            Decl::Proc(Proc {
                                kind: (None, "I64".to_string()),
                                body: vec![Instr::Expr(Expr::Prim(I64(-1)))],
                            })
                        ),
                    ]
                    .into_iter()
                    .collect()
                }
            ))
        );
    }

    #[test]
    fn proc_with_call() {
        assert_eq!(
            ast("proc main -> Void is\n   dump(2021)\n end"),
            Ok((
                "",
                AST {
                    decls: vec![(
                        "main".to_string(),
                        Decl::Proc(Proc {
                            kind: (None, "Void".to_string()),
                            body: vec![Instr::Expr(Expr::Call(Call {
                                proc_name: "dump".to_string(),
                                args: Some(vec![Expr::Prim(I64(2021))])
                            }))],
                        })
                    )]
                    .into_iter()
                    .collect()
                }
            ))
        );
    }

    #[test]
    fn proc_with_bind() {
        assert_eq!(
            ast("proc main -> Void is\n  let number: I8 = 42\n end"),
            Ok((
                "",
                AST {
                    decls: vec![(
                        "main".to_string(),
                        Decl::Proc(Proc {
                            kind: (None, "Void".to_string()),
                            body: vec![Instr::Bind(Bind {
                                id: "number".to_string(),
                                ty: "I8".to_string(),
                                expr: Expr::Prim(I64(42))
                            })],
                        })
                    )]
                    .into_iter()
                    .collect()
                }
            ))
        );
    }
}

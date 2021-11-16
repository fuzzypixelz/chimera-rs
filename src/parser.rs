/*
    This is a prototyping parser for Woland.

    A. Reserved keywords.

      let   end
      true  false
      and   or    not
      if    then  elif else
      do    loop  for  while in do break
      match with
      type  actor

    B. Other tokens.

      : -> ~ =

    C. Grammar.

      prog  -> decl*

      decl  -> type | function | actor

      func  -> 'let' name ':' kind ('~' | '=') body

      kind  -> (name ':' name '->')* name

      body  -> instr* 'end'
            |  instr newline

      instr -> expr newline
            |  bind
            |  cond
            |  loop

      expr  -> prim
            |  name
            |  call

      prim  -> string | integer | boolean

      call  -> name expr*

      bind  -> 'let' 'mut'? name ':' name ('~' | '=') expr newline

      cond  -> 'if' expr 'then' body ('elif' exp 'then' body)* ('else' body)? 'end'

      loop  -> 'loop' expr 'do' body 'end'
*/

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, i64, multispace0, newline, space0},
    combinator::{into, opt, recognize, verify},
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};

use crate::ast::*;

pub fn ast(input: &str) -> IResult<&str, AST> {
    let (input, decls) = many1(func)(input)?;
    Ok((
        input,
        AST {
            decls: decls.into_iter().collect(),
        },
    ))
}

pub fn func(input: &str) -> IResult<&str, (String, Decl)> {
    let (input, _) = ws(tag("let"))(input)?;
    let (input, name) = ws(name)(input)?;
    let (input, kind) = ws(kind)(input)?;
    let (input, _) = alt((ws(tag("~")), ws(tag("="))))(input)?;
    let (input, body) = many1(ws(instr))(input)?;
    let (input, _) = ws(tag("end"))(input)?;
    Ok((input, (name.to_string(), Decl::Func(Func { kind, body }))))
}

fn kind(input: &str) -> IResult<&str, Kind> {
    let (input, _) = ws(char(':'))(input)?;
    let (input, params) = many0(terminated(ws(name_typed), ws(tag("->"))))(input)?;
    let (input, ret) = ws(name)(input)?;
    Ok((
        input,
        Kind {
            params: params
                .iter()
                .map(|(s, t)| (s.to_string(), t.to_string()))
                .collect(),
            ret: ret.to_string(),
        },
    ))
}

fn instr(input: &str) -> IResult<&str, Instr> {
    let (input, result) = alt((terminated(into(expr), newline), ws(bind)))(input)?;
    Ok((input, result))
}

impl From<Expr> for Instr {
    fn from(expr: Expr) -> Self {
        Instr::Expr(expr)
    }
}

fn expr(input: &str) -> IResult<&str, Expr> {
    // None of the alt inputs show consume multispace!
    let (input, expr) = alt((prim, into(name), call))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, expr))
}

impl From<&str> for Expr {
    fn from(str: &str) -> Self {
        Expr::Name(str.to_string())
    }
}

fn prim(input: &str) -> IResult<&str, Expr> {
    let (input, int) = i64(input)?;
    use Prim::*;
    Ok((input, Expr::Prim(I64(int))))
}

fn name(input: &str) -> IResult<&str, &str> {
    verify(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| !vec!["let", "end"].contains(&s),
    )(input)
}

fn name_typed(input: &str) -> IResult<&str, (&str, &str)> {
    delimited(
        opt(ws(char('('))),
        separated_pair(ws(name), ws(char(':')), name),
        opt(ws(char(')'))),
    )(input)
}

fn call(input: &str) -> IResult<&str, Expr> {
    // TODO: fix call.
    let (input, _) = char('@')(input)?;
    let (input, func) = name(input)?;
    let (input, _) = space0(input)?;
    let (input, args) = many0(terminated(expr, space0))(input)?;
    Ok((
        input,
        Expr::Call(Call {
            func_name: func.to_string(),
            args: args,
        }),
    ))
}

fn bind(input: &str) -> IResult<&str, Instr> {
    let (input, _) = ws(tag("let"))(input)?;
    let (input, (id, ty)) = ws(name_typed)(input)?;
    let (input, _) = alt((ws(tag("~")), ws(tag("="))))(input)?;
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

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    terminated(inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Prim::*;

    #[test]
    fn basic_literal() {
        assert_eq!(prim("42"), Ok(("", Expr::Prim(I64(42)))));
    }

    #[test]
    fn basic_call() {
        assert_eq!(
            instr("@dump 666 my_favourite_number\n"),
            Ok((
                "",
                Instr::Expr(Expr::Call(Call {
                    func_name: "dump".to_string(),
                    args: vec![
                        Expr::Prim(I64(666)),
                        Expr::Name("my_favourite_number".to_string())
                    ]
                }))
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
        assert_eq!(name_typed("x: I64"), Ok(("", ("x", "I64"))));
    }

    #[test]
    fn kind_no_args() {
        assert_eq!(
            kind(": I64"),
            Ok((
                "",
                Kind {
                    params: vec![],
                    ret: "I64".to_string()
                }
            ))
        )
    }

    #[test]
    fn kind_two_args() {
        assert_eq!(
            kind(": (x: I64) -> (y: I64) -> I64"),
            Ok((
                "",
                Kind {
                    params: vec![
                        ("x".to_string(), "I64".to_string()),
                        ("y".to_string(), "I64".to_string())
                    ],
                    ret: "I64".to_string()
                }
            ))
        )
    }

    #[test]
    fn basic_func() {
        assert_eq!(
            func("let main: I32 ~\n    -1\n end"),
            Ok((
                "",
                (
                    "main".to_string(),
                    Decl::Func(Func {
                        kind: Kind {
                            params: vec![],
                            ret: "I32".to_string()
                        },
                        body: vec![Instr::Expr(Expr::Prim(I64(-1)))],
                    })
                )
            ))
        )
    }

    #[test]
    fn ast_two_procs() {
        assert_eq!(
            ast("let whatever: I8 =\n   0\n end\n\nlet main: I64 ~\n   -1\n end"),
            Ok((
                "",
                AST {
                    decls: vec![
                        (
                            "whatever".to_string(),
                            Decl::Func(Func {
                                kind: Kind {
                                    params: vec![],
                                    ret: "I8".to_string()
                                },
                                body: vec![Instr::Expr(Expr::Prim(I64(0)))],
                            })
                        ),
                        (
                            "main".to_string(),
                            Decl::Func(Func {
                                kind: Kind {
                                    params: vec![],
                                    ret: "I64".to_string()
                                },
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
            ast("let nothing: Void ~\n   @dump 2021\n end"),
            Ok((
                "",
                (AST {
                    decls: vec![(
                        "nothing".to_string(),
                        Decl::Func(Func {
                            kind: Kind {
                                params: vec![],
                                ret: "Void".to_string()
                            },
                            body: vec![Instr::Expr(Expr::Call(Call {
                                func_name: "dump".to_string(),
                                args: vec![Expr::Prim(I64(2021))]
                            }))],
                        })
                    )]
                    .into_iter()
                    .collect()
                })
            ))
        );
    }

    #[test]
    fn proc_with_bind() {
        assert_eq!(
            ast("let main: Void ~\n  let number: I8 = 42\n end"),
            Ok((
                "",
                AST {
                    decls: vec![(
                        "main".to_string(),
                        Decl::Func(Func {
                            kind: Kind {
                                params: vec![],
                                ret: "Void".to_string()
                            },
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

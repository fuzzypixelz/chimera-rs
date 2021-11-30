pub type Spanned<'input> = Result<(usize, Tok<'input>, usize), LexicalError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Tok<'input> {
    Name(&'input str),
    Operator(&'input str),

    IntLiteral(i64),

    Ellipsis,

    Let,
    End,
    Type,

    True,
    False,
    If,
    Then,
    Elif,
    Else,

    Loop,

    Break,

    Colon,
    Arrow,
    Pipe,
    Comma,
    Equal,
    Tilde,

    LParen,
    RParen,
    LBrack,
    RBrack,

    Newline,
}

pub static RESERVED_NAMES: phf::Map<&'static str, Tok> = phf::phf_map! {
    "let"   => Tok::Let,
    "end"   => Tok::End,
    "type"  => Tok::Type,
    "true"  => Tok::True,
    "false" => Tok::False,
    "if"    => Tok::If,
    "then"  => Tok::Then,
    "elif"  => Tok::Elif,
    "else"  => Tok::Else,
    "loop"  => Tok::Loop,
    "break" => Tok::Break,
};

pub static RESERVED_SYMBOLS: phf::Map<&'static str, Tok> = phf::phf_map! {
    "..." => Tok::Ellipsis,
    ":"   => Tok::Colon,
    "->"  => Tok::Arrow,
    "|"   => Tok::Pipe,
    "="   => Tok::Equal,
    "~"   => Tok::Tilde,
};

#[derive(Copy, Clone, Debug)]
pub enum LexicalError {
    InvalidSyntax,
}

use std::iter::Peekable;
use std::str::CharIndices;
use std::str::FromStr;

pub struct Lexer<'input> {
    chars: Peekable<CharIndices<'input>>,
    input: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            chars: input.char_indices().peekable(),
            input,
        }
    }

    fn take_while<F>(
        &mut self,
        start: usize,
        mut predicate: F,
    ) -> (usize, &'input str)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(&(end, c)) = self.chars.peek() {
            if !predicate(c) {
                return (end, &self.input[start..end]);
            } else {
                self.chars.next();
            }
        }
        // Reached EOF without the predicate failling.
        let end = self.input.len();
        (end, &self.input[start..end])
    }

    fn name(&mut self, start: usize) -> Spanned<'input> {
        let (end, src) =
            self.take_while(start, |c| c.is_alphanumeric() || c == '_');
        let token = if RESERVED_NAMES.contains_key(src) {
            RESERVED_NAMES[src]
        } else {
            Tok::Name(src)
        };

        Ok((start, token, end))
    }

    fn operator(&mut self, start: usize) -> Spanned<'input> {
        let (end, src) =
            self.take_while(start, |c| "~!@#$%^&*-+=|:;?<>.,\\".contains(c));
        let token = if RESERVED_SYMBOLS.contains_key(src) {
            RESERVED_SYMBOLS[src]
        } else {
            Tok::Operator(src)
        };

        Ok((start, token, end))
    }

    fn integer(&mut self, start: usize) -> Spanned<'input> {
        let (end, src) = self.take_while(start, |c| c.is_numeric());
        let int = i64::from_str(src).unwrap();
        Ok((start, Tok::IntLiteral(int), end))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&(start, c)) = self.chars.peek() {
            return match c {
                c if c.is_whitespace() => {
                    let (end, src) =
                        self.take_while(start, |c| c.is_whitespace());
                    if src.contains('\n') {
                        Some(Ok((start, Tok::Newline, end)))
                    } else {
                        continue;
                    }
                }
                // TODO: This convenience function includes
                // Unicode characters too, replace it or keep
                // this in the language.
                c if c.is_numeric() => Some(self.integer(start)),
                c if c.is_alphabetic() => Some(self.name(start)),
                '(' => {
                    self.chars.next();
                    Some(Ok((start, Tok::LParen, start + 1)))
                }
                ')' => {
                    self.chars.next();
                    Some(Ok((start, Tok::RParen, start + 1)))
                }
                '{' => {
                    self.chars.next();
                    Some(Ok((start, Tok::LBrack, start + 1)))
                }
                '}' => {
                    self.chars.next();
                    Some(Ok((start, Tok::RBrack, start + 1)))
                }
                ',' => {
                    self.chars.next();
                    Some(Ok((start, Tok::Comma, start + 1)))
                }
                _ => Some(self.operator(start)),
            };
        }
        None
    }
}

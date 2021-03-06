use std::fmt::Display;
use std::iter::Peekable;
use std::str::CharIndices;
use std::str::FromStr;

use crate::error::LexicalError;

pub type Spanned<'input> = Result<(usize, Tok<'input>, usize), LexicalError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Tok<'input> {
    Name(&'input str),
    TypeName(&'input str),

    Operator(&'input str),

    IntLiteral(i64),
    StrLiteral(&'input str),
    CharLiteral(char),

    Ellipsis,

    At,
    Dot,

    Mod,
    Let,
    Do,
    End,
    Data,
    Forall,

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
    Hash,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,

    Newline,
}

pub static RESERVED_NAMES: phf::Map<&'static str, Tok> = phf::phf_map! {
    "mod"       => Tok::Mod,
    "let"       => Tok::Let,
    "do"        => Tok::Do,
    "end"       => Tok::End,
    "data"      => Tok::Data,
    "forall"    => Tok::Forall,
    "true"      => Tok::True,
    "false"     => Tok::False,
    "if"        => Tok::If,
    "then"      => Tok::Then,
    "elif"      => Tok::Elif,
    "else"      => Tok::Else,
    "loop"      => Tok::Loop,
    "break"     => Tok::Break,
};

pub static RESERVED_SYMBOLS: phf::Map<&'static str, Tok> = phf::phf_map! {
    "..." => Tok::Ellipsis,
    ":"   => Tok::Colon,
    "->"  => Tok::Arrow,
    "|"   => Tok::Pipe,
    "="   => Tok::Equal,
    "~"   => Tok::Tilde,
    "@"   => Tok::At,
    "."   => Tok::Dot,
    "#"   => Tok::Hash,
};

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

    fn take_while<F>(&mut self, start: usize, mut predicate: F) -> (usize, &'input str)
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
        let (end, src) = self.take_while(start, |c| c.is_alphanumeric() || c == '_');
        let token = if RESERVED_NAMES.contains_key(src) {
            RESERVED_NAMES[src]
        } else {
            Tok::Name(src)
        };

        Ok((start, token, end))
    }

    fn type_name(&mut self, start: usize) -> Spanned<'input> {
        let (end, src) = self.take_while(start, |c| c.is_alphanumeric() || c == '_');
        let token = if RESERVED_NAMES.contains_key(src) {
            RESERVED_NAMES[src]
        } else {
            Tok::TypeName(src)
        };

        Ok((start, token, end))
    }

    fn operator(&mut self, start: usize) -> Spanned<'input> {
        let (end, src) = self.take_while(start, |c| "/~!@#$%^&*-+=|:;?<>.,\\".contains(c));
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

    fn string(&mut self, start: usize) -> Spanned<'input> {
        self.chars.next(); // Consume the opening double quotes.
        let (end, src) = self.take_while(start + 1, |b| b != '"');
        self.chars.next(); // Consume the ending double quotes.
        Ok((start, Tok::StrLiteral(src), end))
    }

    fn character(&mut self, start: usize) -> Spanned<'input> {
        self.chars.next();
        let (_, src) = self
            .chars
            .next()
            .ok_or(LexicalError::InvalidSyntax)?;
        let token = if src == '\\' {
            let (_, escape) = self
                .chars
                .next()
                .ok_or(LexicalError::InvalidSyntax)?;
            Tok::CharLiteral(match escape {
                '\\' => '\\',
                '\'' => '\'',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err(LexicalError::InvalidSyntax)
            })
        } else {
            Tok::CharLiteral(src)
        };
        self.chars.next();
        Ok((start, token, start + 3))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&(start, c)) = self.chars.peek() {
            return match c {
                c if c.is_whitespace() => {
                    let (end, src) = self.take_while(start, |c| c.is_whitespace());
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
                c if c.is_uppercase() => Some(self.type_name(start)),
                c if c.is_lowercase() || c == '_' => Some(self.name(start)),
                '\'' => Some(self.character(start)),
                '"' => Some(self.string(start)),
                '-' => {
                    self.chars.next(); // Consume the first hyphen
                    if let Some(&(start, '-')) = self.chars.peek() {
                        self.take_while(start, |c| c != '\n');
                        // Also consume all newlines that follow, they are irrelevant
                        // for the syntax as the context is a comment.
                        self.take_while(start, |c| c.is_whitespace());
                        continue;
                    } else {
                        // HACK: this is worse than it looks, as it uses the fact
                        // that the already consumed hyphen will be part of an
                        // operator, hence why we allow ourselves to start
                        // the search done in the `take_while` call of operator.
                        Some(self.operator(start))
                    }
                }
                // NOTE: candidates for multiline comments:
                //  1. {- ... -} Haskell
                //  2. (* ... *) AppleScript, *ML, Pascal
                //  3. --[[ ... ]]-- Lua
                //  4. -{ ... }- Passerine
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
                    Some(Ok((start, Tok::LBrace, start + 1)))
                }
                '}' => {
                    self.chars.next();
                    Some(Ok((start, Tok::RBrace, start + 1)))
                }
                '[' => {
                    self.chars.next();
                    Some(Ok((start, Tok::LBrack, start + 1)))
                }
                ']' => {
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

impl<'input> Display for Tok<'input> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

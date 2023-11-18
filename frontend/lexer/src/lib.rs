#![feature(iterator_try_collect)]

use std::ops::Range;

pub use logos::{Lexer, Logos, SpannedIter};

#[derive(Logos, Debug, Clone)]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex(r#""[^"]*""#, |lex| { let slice = lex.slice(); slice[1..slice.len()-1].to_owned() })]
    StringLiteral(String),
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("@")]
    At,
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("Lexer error at {}..{}", self.0.start, self.0.end)]
#[diagnostic(code = "tangi::lexer")]
pub struct LexerError(pub Range<usize>);

pub struct Help {
    j: <Vec<(Token, Range<usize>)> as IntoIterator>::IntoIter,
}

impl Help {
    pub fn new(input: &str) -> Result<Self, Vec<LexerError>> {
        let mut lexer = Token::lexer(input).spanned();

        let mut help = vec![];
        let mut guh = vec![];
        while let Some(s) = lexer.next() {
            match s {
                (Ok(j), span) => guh.push((j, span)),
                (Err(()), span) => help.push(LexerError(span))
            }
        };

        if help.is_empty() {
            Ok(Self { j: guh.into_iter() })
        } else {
            Err(help)
        }
    }
}

impl Iterator for Help {
    type Item = (Token, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        self.j.next()
    }
}

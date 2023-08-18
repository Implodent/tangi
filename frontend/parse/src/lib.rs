mod parsers;
use std::{
        fmt::{Debug, Display},
        iter::{Cloned, Enumerate},
        slice::Iter,
};

use logos::{Lexer, Logos, Span};

#[derive(Debug)]
pub struct WithSpan<T>(pub T, pub Span);
impl<T: Debug> Display for WithSpan<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?} @ {}..{}", self.0, self.1.start, self.1.end)
        }
}
impl<T: Clone> Clone for WithSpan<T> {
        fn clone(&self) -> Self {
                Self(self.0.clone(), self.1.clone())
        }
        fn clone_from(&mut self, source: &Self) {
                self.0 = source.0.clone();
                self.1 = source.1;
        }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
pub enum Token {
        #[token("fn")]
        KeywordFn,
        #[token("const")]
        KeywordConst,
        #[token("=")]
        PunctEq,
        #[regex(r"[A-Za-z_][A-Za-z_0-9]*", |lex| lex.slice().to_owned())]
        Ident(String),
        #[token("(")]
        OpenParen,
        #[token(")")]
        CloseParen,
        #[token(":")]
        Colon,
        #[token(",")]
        Comma,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
        #[error("invalid token, expected {expected}, actual: {actual:?}")]
        InvalidToken { expected: String, actual: Token },
        #[error("unexpected end of file")]
        UnexpectedEof,
}

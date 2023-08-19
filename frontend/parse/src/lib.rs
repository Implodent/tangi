mod parsers;
use std::{
        borrow::Cow,
        fmt::{Debug, Display},
        num::ParseIntError,
};

use chumsky::{error::SimpleReason, prelude::Simple, Parser, Stream};
use logos::{Logos, Span, SpannedIter};

pub fn fmt_debug<T: Debug>(t: T) -> String {
        format!("{t:?}")
}

#[derive(Debug)]
pub struct WithSpan<T>(pub T, pub Span);
impl<T> WithSpan<T> {
        pub fn map<R>(self, f: impl Fn(T) -> R) -> WithSpan<R> {
                WithSpan(f(self.0), self.1)
        }
}
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
#[logos(skip r"[\n\r\t\w ]*")]
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
        #[error("invalid token, expected {expected:?}, actual: {actual:?}")]
        InvalidToken {
                expected: Vec<Cow<'static, str>>,
                actual: Token,
        },
        #[error("unexpected end of file")]
        UnexpectedEof,
        #[error("integer parse error: {_0}")]
        IntError(#[from] ParseIntError),
        #[error("{_0}")]
        Other(Cow<'static, str>),
}

pub fn parse<'H>(src: &'H str) -> Result<tangic_common::ast::File, Vec<Error>> {
        let eoi = src.len()..src.len() + 1;
        parsers::help().parse(Stream::from_iter(
                eoi,
                Token::lexer(src)
                        .spanned()
                        .map(|result| (result.0.unwrap(), result.1)),
        ))
}

#[derive(Debug, Default)]
pub struct Error {
        pub labels: Vec<String>,
        pub errors: Vec<WithSpan<ParseError>>,
}

impl Error {
        pub fn new(span: Span, e: ParseError) -> Self {
                Self {
                        labels: vec![],
                        errors: vec![WithSpan(e, span)],
                }
        }
        pub fn other(span: Span, reason: Cow<'static, str>) -> Self {
                Self {
                        labels: vec![],
                        errors: vec![WithSpan(ParseError::Other(reason), span)],
                }
        }
        pub fn merge_(mut self, other: Self) -> Self {
                self.errors.extend(other.errors);
                self.labels.extend(other.labels);
                self
        }
}

impl chumsky::Error<Token> for Error {
        type Span = Span;
        type Label = String;

        fn expected_input_found<Iter: IntoIterator<Item = Option<Token>>>(
                span: Self::Span,
                expected: Iter,
                found: Option<Token>,
        ) -> Self {
                Self {
                        errors: vec![WithSpan(
                                match found {
                                        Some(found) => ParseError::InvalidToken {
                                                expected: expected
                                                        .into_iter()
                                                        .map(|opt| {
                                                                opt.map(fmt_debug)
                                                                        .map(Cow::Owned)
                                                                        .unwrap_or(Cow::Borrowed(
                                                                                "end of file",
                                                                        ))
                                                        })
                                                        .collect(),
                                                actual: found,
                                        },
                                        None => ParseError::UnexpectedEof,
                                },
                                span,
                        )],
                        labels: vec![],
                }
        }
        fn merge(self, other: Self) -> Self {
                self.merge_(other)
        }
        fn with_label(mut self, label: Self::Label) -> Self {
                self.labels.push(label);
                self
        }
}

impl chumsky::Error<char> for Error {
        type Span = Span;
        type Label = String;

        fn expected_input_found<Iter: IntoIterator<Item = Option<char>>>(
                span: Self::Span,
                expected: Iter,
                found: Option<char>,
        ) -> Self {
                Self::new(
                        span,
                        match found {
                                Some(found) => ParseError::InvalidToken {
                                        expected: expected
                                                .into_iter()
                                                .map(|t| {
                                                        t.map(|cr| cr.to_string())
                                                                .map(Cow::Owned)
                                                                .unwrap_or(Cow::Borrowed("EOF"))
                                                })
                                                .collect(),
                                        actual: found,
                                },
                                None => ParseError::UnexpectedEof,
                        },
                )
        }
        fn merge(self, other: Self) -> Self {
                self.merge_(other)
        }
        fn with_label(mut self, label: Self::Label) -> Self {
                self.labels.push(label);
                self
        }
}

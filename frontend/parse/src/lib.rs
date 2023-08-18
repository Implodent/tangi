mod parsers;
use std::{
        borrow::Cow,
        fmt::{Debug, Display},
};

use chumsky::{Parser, Stream};
use logos::{Logos, Span, SpannedIter};

pub fn fmt_debug<T: Debug>(t: T) -> String {
        format!("{t:?}")
}

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
        #[error("invalid token, expected {expected}, actual: {actual:?}")]
        InvalidToken {
                expected: Cow<'static, str>,
                actual: Token,
        },
        #[error("unexpected end of file")]
        UnexpectedEof,
}

pub fn parse<'H>(src: &'H str) -> Result<tangic_common::ast::File, Vec<WithSpan<ParseError>>> {
        let eoi = src.len()..src.len() + 1;
        parsers::help().parse(Stream::from_iter(
                eoi,
                Token::lexer(src)
                        .spanned()
                        .map(|result| (result.0.unwrap(), result.1)),
        ))
}

#[derive(Debug)]
pub struct Error {
        pub labels: Vec<String>,
        pub parsed: Vec<WithSpan<ParseError>>,
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
                        parsed: vec![WithSpan(
                                match found {
                                        Some(found) => ParseError::InvalidToken {
                                                expected: expected
                                                        .into_iter()
                                                        .next()
                                                        .map(fmt_debug)
                                                        .map(Cow::Owned)
                                                        .unwrap_or(Cow::Borrowed("end of file")),
                                                actual: found,
                                        },
                                        None => ParseError::UnexpectedEof,
                                },
                                span,
                        )],
                        labels: vec![],
                }
        }
        fn merge(mut self, other: Self) -> Self {
                self.parsed.extend(other.parsed);
                self.labels.extend(other.labels);
                self
        }
        fn with_label(mut self, label: Self::Label) -> Self {
                self.labels.push(label);
                self
        }
}

mod parsers;
use std::{
        borrow::Cow,
        collections::HashMap,
        fmt::{Debug, Display},
        num::ParseIntError,
};

use aott::{
        extra,
        prelude::{InputType, Parser, ParserExtras},
        stream::Stream,
};
use logos::{Lexer, Logos, Span, SpannedIter};
use tangic_common::ast::ExprNumber;
use tracing::{debug, info};

pub fn fmt_debug<T: Debug>(t: T) -> String {
        format!("{t:?}")
}

#[derive(Debug, PartialEq, Eq)]
pub struct WithSpan<T>(pub T, pub Span);
impl<T> WithSpan<T> {
        pub fn map<R>(self, f: impl Fn(T) -> R) -> WithSpan<R> {
                WithSpan(f(self.0), self.1)
        }
        pub fn with_span(span: Span) -> impl Fn(T) -> Self {
                move |t| Self(t, span.clone())
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
                self.1 = source.1.clone();
        }
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = ParseError)]
#[logos(skip r"[\n\r\t\f ]+")]
pub enum Token {
        #[token("fn")]
        KeywordFn,
        #[token("const")]
        KeywordConst,
        #[token("return")]
        KeywordReturn,
        #[token("=")]
        PunctEq,
        #[regex(r"[A-Za-z_][A-Za-z_0-9]*", |lex| lex.slice().to_owned())]
        Ident(String),
        #[token("(")]
        OpenParen,
        #[token(")")]
        CloseParen,
        #[token("[")]
        OpenBracket,
        #[token("]")]
        CloseBracket,
        #[token("{")]
        OpenCurly,
        #[token("}")]
        CloseCurly,
        #[token(":")]
        Colon,
        #[token(",")]
        Comma,
        #[token("?")]
        Question,
        #[token("!")]
        Bang,
        #[token(";")]
        Semi,
        #[token("'")]
        SingleQuote,
        #[token(".")]
        Dot,
        #[regex(r"(0b|0x|0o)?\d+([ui](\d\d?\d?)|size)?", |lex| parsers::parse_number.parse(lex.slice()).map_err(|e| e.error))]
        Number(ExprNumber),
}

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum ParseError {
        #[error("invalid token, expected {expected:?}, found {found:?}")]
        InvalidToken { expected: Vec<Token>, found: Token },
        #[error("unexpected end of file")]
        UnexpectedEof,
        #[error("expected end of file, found {_0:?}")]
        ExpectedEofFound(Token),
        #[error("integer parse error: {_0}")]
        IntError(#[from] ParseIntError),
        #[error("{_0}")]
        Other(Cow<'static, str>),
        #[error("parsing error: {_0:?}")]
        CharParse(#[from] aott::extra::Simple<char>),
}

#[derive(thiserror::Error, Debug, PartialEq, Clone)]
#[error("{error} (at {}..{})", span.start, span.end)]
pub struct Error {
        pub span: Span,
        pub error: ParseError,
}
impl Default for ParseError {
        fn default() -> Self {
                Self::Other(Cow::Borrowed("Fuck you"))
        }
}

impl aott::error::Error<Tokens> for Error {
        type Span = Span;
        fn expected_eof_found(span: Span, found: aott::MaybeRef<'_, Token>) -> Self {
                Self {
                        span,
                        error: ParseError::ExpectedEofFound(found.into_clone()),
                }
        }
        fn expected_token_found(
                span: Span,
                expected: Vec<Token>,
                found: aott::MaybeRef<'_, Token>,
        ) -> Self {
                Self {
                        span,
                        error: ParseError::InvalidToken {
                                expected,
                                found: found.into_clone(),
                        },
                }
        }
        fn unexpected_eof(span: Span, _expected: Option<Vec<Token>>) -> Self {
                Self {
                        span,
                        error: ParseError::UnexpectedEof,
                }
        }
}
impl<'a> aott::error::Error<&'a str> for Error {
        type Span = Span;
        fn expected_eof_found(
                span: Span,
                found: aott::MaybeRef<'_, <&'a str as InputType>::Token>,
        ) -> Self {
                Self {
                        span: span.clone(),
                        error: ParseError::CharParse(<extra::Simple<char> as aott::error::Error<
                                &'a str,
                        >>::expected_eof_found(
                                span, found
                        )),
                }
        }
        fn expected_token_found(
                span: Span,
                expected: Vec<<&'a str as InputType>::Token>,
                found: aott::MaybeRef<'_, <&'a str as InputType>::Token>,
        ) -> Self {
                Self {
                        span: span.clone(),
                        error: ParseError::CharParse(<extra::Simple<char> as aott::error::Error<
                                &'a str,
                        >>::expected_token_found(
                                span, expected, found
                        )),
                }
        }
        fn unexpected_eof(
                span: Span,
                expected: Option<Vec<<&'a str as InputType>::Token>>,
        ) -> Self {
                Self {
                        span: span.clone(),
                        error: ParseError::CharParse(<extra::Simple<char> as aott::error::Error<
                                &'a str,
                        >>::unexpected_eof(
                                span, expected
                        )),
                }
        }
}

#[derive(Debug, Clone, Copy)]
pub struct Extra;
impl ParserExtras<Tokens> for Extra {
        type Error = Error;
        type Context = ();
}

impl<'a> ParserExtras<&'a str> for Extra {
        type Context = ();
        type Error = Error;
}

type Tokens = Stream<Box<dyn Iterator<Item = Token>>>;

#[tracing::instrument(err(Debug), ret)]
pub fn parse(src: &str) -> Result<tangic_common::ast::File, Vec<Error>> {
        let eoi = src.len()..src.len() + 1;
        let src_leaked: &'static mut str = Box::leak(src.to_owned().into_boxed_str());
        let mut errors = vec![];
        let mut tokens = vec![];
        for (n, (token, span)) in Token::lexer(src).spanned().enumerate() {
                match token {
                        Ok(token) => {
                                debug!(%span.start, %span.end, ?token);

                                tokens.insert(n, (token, span));
                        }
                        Err(error) => errors.push(Error { span, error }),
                }
        }
        if !errors.is_empty() {
                return Err(errors);
        }

        let result = parsers::file.parse(Stream::from_iter(
                <Token as Logos<'static>>::lexer(src_leaked).map(unwrap_unchecked as fn(_) -> _),
        )
        .boxed());
        match result {
                Ok(ast) => Ok(ast),
                Err(mut e) => {
                        e.span = tokens.get(e.span.start).unwrap().1.start
                                ..tokens.get(e.span.end).unwrap().1.end;
                        errors.push(e);
                        Err(errors)
                }
        }
}

fn unwrap_unchecked(result: Result<Token, ParseError>) -> Token {
        unsafe { result.unwrap_unchecked() }
}

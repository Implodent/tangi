use std::fmt::{Debug, Display};

use logos::{Lexer, Logos, Span};
use tangic_common::ast;

#[derive(Debug)]
pub struct WithSpan<T>(pub T, pub Span);
impl<T: Debug> Display for WithSpan<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?} @ {}..{}", self.0, self.1.start, self.1.end)
        }
}
impl<T: Clone> Clone for WithSpan<T> {
        fn clone(&self) -> Self {
                Self(self.0.clone(), *self.1)
        }
        fn clone_from(&mut self, source: &Self) {
                self.0 = source.0.clone();
                self.1 = source.1;
        }
}
impl<T: Copy> Copy for WithSpan<T> {}

#[derive(Logos, Debug)]
pub enum Token {
        #[token("fn")]
        KeywordFn,
        #[token("const")]
        KeywordConst,
        #[token("=")]
        PunctEq,
}

pub struct Parser<'h> {
        iter: logos::Lexer<'h, Token>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
        InvalidToken {
                expected: &'static str,
                actual: Token,
        },
        UnexpectedEof,
        Lexer(#[from] <Token as Logos>::Error),
}

type Result<T, E = ParseError> = core::result::Result<T, WithSpan<E>>;

impl<'h> Parser<'h> {
        pub fn new(src: &'h <Token as Logos>::Source) -> Self {
                Self {
                        iter: Token::lexer(src),
                }
        }

        fn spanned<T>(&self, thing: T) -> WithSpan<T> {
                WithSpan(thing, self.iter.span())
        }

        pub fn ask_file(&mut self) -> Result<ast::File> {
                Err(self.spanned(ParseError::InvalidToken {
                        expected: "h",
                        actual: self.next_token()?,
                }))
        }

        fn next_token(&mut self) -> Result<Token> {
                self.iter
                        .next()
                        .ok_or_else(|| self.spanned(ParseError::UnexpectedEof))
                        .and_then(|result| {
                                Ok(result.map_err(ParseError::from).map_err(self.spanned))
                        })
        }
}

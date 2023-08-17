use logos::{Lexer, Logos};
use tangic_common::ast;

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
    iter: logos::SpannedIter<'h, Token>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    InvalidToken,
}

type Result<T, E = Error> = core::result::Result<T, E>;

impl<'h> Parser<'h> {
    pub fn new(tokens: Lexer<'h, Token>) -> Self {
        Self {
            iter: tokens.spanned(),
        }
    }

    pub fn ask_file(&mut self) -> Result<ast::File> {}
}

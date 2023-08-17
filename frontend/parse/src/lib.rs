extern crate tangic_ast as ast;
use logos::{Lexer, Logos};

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

impl<'h> Parser<'h> {
    pub fn new(tokens: Lexer<'h, Token>) -> Self {
        Self {
            iter: tokens.spanned(),
        }
    }

    pub fn ask_file(&mut self) -> ast::File {}
}

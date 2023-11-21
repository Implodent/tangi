#![feature(iterator_try_collect)]

use std::ops::Range;

pub use logos::{Lexer, Logos, SpannedIter};
use miette::SourceSpan;

#[derive(Logos, Debug, Clone, PartialEq, derive_more::Display)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[display(fmt = "an identifier \"{_0}\"")]
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[display(fmt = "a string literal \"{_0}\"")]
    #[regex(r#""[^"]*""#, |lex| { let slice = lex.slice(); slice[1..slice.len()-1].to_owned() })]
    StringLiteral(String),
    #[display(fmt = "an opening curly brace {{")]
    #[token("{")]
    OpenCurly,
    #[display(fmt = "a closing curly brace }}")]
    #[token("}")]
    CloseCurly,
    #[display(fmt = "opening parentheses `(`")]
    #[token("(")]
    OpenParen,
    #[display(fmt = "closing parentheses `)`")]
    #[token(")")]
    CloseParen,
    #[display(fmt = "a dot .")]
    #[token(".")]
    Dot,
    #[display(fmt = "a comma ,")]
    #[token(",")]
    Comma,
    #[display(fmt = "an @ sign")]
    #[token("@")]
    At,
    #[display(fmt = "an = sign")]
    #[token("=")]
    Eq,
    #[display(fmt = "an exclamation mark")]
    #[token("!")]
    Excl,
    #[display(fmt = "the `const` keyword")]
    #[token("const")]
    KwConst,
    #[display(fmt = "the `pub` keyword")]
    #[token("pub")]
    KwPub
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TokenKind {
    #[error("an identifier")]
    Identifier,
    #[error("a string literal")]
    StringLiteral,
    #[error("an opening curly brace {{")]
    OpenCurly,
    #[error("a closing curly brace }}")]
    CloseCurly,
    #[error("opening parentheses `(`")]
    OpenParen,
    #[error("closing parentheses `)`")]
    CloseParen,
    #[error("a dot .")]
    Dot,
    #[error("a comma ,")]
    Comma,
    #[error("an @ sign")]
    At,
    #[error("an = sign")]
    Eq,
    #[error("the `const` keyword")]
    KwConst,
    #[error("the `pub` keyword")]
    KwPub,
    #[error("an exclamation mark")]
    Excl
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Self::At => TokenKind::At,
            Self::Comma => TokenKind::Comma,
            Self::OpenCurly => TokenKind::OpenCurly,
            Self::CloseCurly => TokenKind::CloseCurly,
            Self::Dot => TokenKind::Dot,
            Self::Eq => TokenKind::Eq,
            Self::Identifier(_) => TokenKind::Identifier,
            Self::StringLiteral(_) => TokenKind::StringLiteral,
            Self::KwConst => TokenKind::KwConst,
            Self::KwPub => TokenKind::KwPub,
            Self::OpenParen => TokenKind::OpenParen,
            Self::CloseParen => TokenKind::CloseParen,
            Self::Excl => TokenKind::Excl
        }
    }
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("lexer error")]
#[diagnostic(code = "tangi::lexer")]
pub struct LexerError(#[label = "here"] pub SourceSpan);

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
                (Err(()), span) => help.push(LexerError(span.into()))
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.j.size_hint()
    }
}

use std::ops::Range;

use aott::error::LabelError;
use miette::{SourceCode, SourceSpan};
use tangic_lexer::{LexerError, Token, TokenKind};

use crate::adapters::TokenStream;

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum ParserError {
    #[error("{}", if _0.len() == 1 { format!("lexing error: {}", &_0[0]) } else { format!("multiple lexing errors") })]
    #[diagnostic(code(tangi::lexer))]
    Lexer(#[related] Vec<tangic_lexer::LexerError>),
    #[error("unexpected end of input{}", if expected.is_empty() { String::new() } else { format!(", expected {}", expected.iter().map(ToString::to_string).collect::<Vec<String>>().join(" or ")) })]
    #[diagnostic()]
    UnexpectedEof {
        #[label = "more tokens expected here"]
        at: SourceSpan,
        expected: Vec<Expectation>,
    },
    #[error("expected {} but found {found}", if expectation.is_empty() { format!("nothing (???)")} else { expectation.iter().map(ToString::to_string).collect::<Vec<String>>().join(" or ") })]
    #[diagnostic(code(tangic::parser::expected))]
    Expected {
        expectation: Vec<Expectation>,
        found: Token,
        #[label = "here"]
        at: SourceSpan,
    },
}

#[derive(thiserror::Error, Debug, Clone, miette::Diagnostic)]
pub enum Expectation {
    #[error("{_0}")]
    #[diagnostic()]
    Kind(TokenKind),
    #[error("{}", _0.iter().map(ToString::to_string).collect::<Vec<String>>().join(" or "))]
    #[diagnostic()]
    AnyOf(Vec<TokenKind>),
    #[error("end of input")]
    #[diagnostic()]
    EndOfInput,
    #[error("a visibility (pub, pub(crate), etc.)")]
    Visibility,
}

impl LabelError<TokenStream, Expectation> for ParserError {
    fn from_label(span: Range<usize>, label: Expectation, last_token: Option<Token>) -> Self {
        Self::Expected {
            expectation: vec![label],
            found: last_token.expect("???"),
            at: span.into(),
        }
    }
}

impl From<Vec<tangic_lexer::LexerError>> for ParserError {
    fn from(err: Vec<tangic_lexer::LexerError>) -> Self {
        Self::Lexer(err)
    }
}

impl aott::error::Error<TokenStream> for ParserError {
    fn unexpected_eof(
        span: <TokenStream as aott::prelude::InputType>::Span,
        expected: Option<Vec<<TokenStream as aott::prelude::InputType>::Token>>,
    ) -> Self {
        Self::UnexpectedEof {
            at: (span.start, span.end).into(),
            expected: expected
                .into_iter()
                .flatten()
                .map(|token| Expectation::Kind(token.kind()))
                .collect(),
        }
    }

    fn expected_eof_found(
        span: <TokenStream as aott::prelude::InputType>::Span,
        found: <TokenStream as aott::prelude::InputType>::Token,
    ) -> Self {
        Self::Expected {
            expectation: vec![Expectation::EndOfInput],
            found,
            at: span.into(),
        }
    }

    fn expected_token_found(
        span: <TokenStream as aott::prelude::InputType>::Span,
        expected: Vec<<TokenStream as aott::prelude::InputType>::Token>,
        found: <TokenStream as aott::prelude::InputType>::Token,
    ) -> Self {
        Self::Expected {
            expectation: expected
                .into_iter()
                .map(|token| Expectation::Kind(token.kind()))
                .collect(),
            found,
            at: span.into(),
        }
    }
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("{}", if _0.len() == 1 { format!("{}", &_0[0]) } else { format!("multiple parsing errors") })]
#[diagnostic()]
pub struct ParserErrors(#[related] pub Vec<ParserError>);

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("parsing failed due to {} error{}", errors.len(), if errors.len().to_string().ends_with("1") { "" } else { "s" })]
pub struct ParserErrorsWithSource<S: SourceCode + std::fmt::Debug> {
    #[related]
    pub errors: Vec<ParserError>,
    #[source_code]
    pub source_code: S,
}

impl From<ParserError> for ParserErrors {
    fn from(value: ParserError) -> Self {
        Self(vec![value])
    }
}

impl From<Vec<LexerError>> for ParserErrors {
    fn from(err: Vec<LexerError>) -> Self {
        Self(vec![ParserError::Lexer(err)])
    }
}

#![feature(try_blocks)]

use adapters::*;
use aott::{pfn_type, prelude::*};

mod adapters;
pub mod error;
use error::*;
use miette::SourceCode;
use tangic_ast as ast;
use tangic_lexer::{Token, TokenKind};

trait Parse {
    fn parse(input: &mut Input<TokenStream, Extra>) -> Result<Self, ParserError>
    where
        Self: Sized;
}

pub fn parse<S: SourceCode + std::fmt::Debug>(
    input: String,
    src: S,
) -> Result<(ast::File, ParserErrorsWithSource<S>), ParserErrorsWithSource<S>> {
    match try {
        let tokens = TokenStream::new(input.clone())?;
        let mut inp = Input::new(&tokens);
        let mut errors = vec![];
        let parsed = file.parse_with(&mut inp); // parse, store a result

        // collect secondary errors
        errors.extend(inp.errors.secondary.drain(..).map(|located| located.err));

        match parsed {
            Ok(real) => Ok((real, ParserErrors(errors))),
            Err(error) => Err({
                errors.push(error);
                ParserErrors(errors)
            }),
        }?
    } {
        Ok((file, ParserErrors(errors))) => Ok((
            file,
            ParserErrorsWithSource {
                errors,
                source_code: src,
            },
        )),
        Err(ParserErrors(errors)) => Err(ParserErrorsWithSource {
            errors,
            source_code: src,
        }),
    }
}

impl Parse for ast::File {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        try {
            let file = Self {
                attributes: attributes(true)(input)?,
                items: vec![],
            };

            end(input)?;

            file
        }
    }
}

#[parser(extras = Extra)]
fn ident(input: TokenStream) -> ast::Ident {
    let before = input.offset;

    // me when diagnostics
    match input.next_or_none() {
        Some(Token::Identifier(ident)) => Ok(ident),
        Some(other_token) => Err(ParserError::Expected {
            expectation: vec![Expectation::Kind(TokenKind::Identifier)],
            found: other_token,
            at: input.span_since(before).into(),
        }),
        None => Err(ParserError::UnexpectedEof {
            expected: vec![Expectation::Kind(TokenKind::Identifier)],
            at: input.span_since(before).into(),
        }),
    }
}

fn attributes(inner: bool) -> pfn_type!(TokenStream, Vec<ast::Attribute>, Extra) {
    move |input| {
        let mut attributes = vec![];

        while let Ok(Token::At) = input.peek() {
            let before = input.save();
            let attr = ast::Attribute::parse(input)?;

            if inner && !attr.inner {
                input.rewind(before);
                break;
            }

            attributes.push(attr);
        }

        Ok(attributes)
    }
}

impl Parse for ast::Attribute {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        try {
            just(Token::At)(input)?;

            let inner = just(Token::At)
                .ignored()
                .optional()
                .parse_with(input)?
                .is_some();

            let name = ident(input)?;

            Self {
                name,
                arguments: vec![],
                inner,
            }
        }
    }
}

impl Parse for ast::Visibility {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        const VIS: &[&str] = &[];

        ident
            .filter(|x| VIS.contains(&x.as_str()), |_| Expectation::Visibility)
            .optional()
            .map(|opt_vis| match opt_vis.as_deref() {
                Some("pub") => ast::Visibility::Public,
                _ => ast::Visibility::Inherited,
            })
            .parse_with(input)
    }
}

impl Parse for ast::Function {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        try {
            Self {
                modifiers: ast::FunctionModifiers::default(),
                vis: input.parse(&Parse::parse)?,
                name: ident(input)?
            }
        }
    }
}

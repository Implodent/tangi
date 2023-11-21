#![feature(try_blocks)]

use adapters::*;
use aott::{pfn_type, prelude::*};

mod adapters;
pub mod error;
use ast::TypeReference;
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
        let parsed = ast::File::parse(&mut inp); // parse, store a result

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
                items: items(input)?,
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
        match input.peek()? {
            Token::KwPub => {
                input.skip()?;
                Ok(Self::Public)
            }
            _ => Ok(Self::Inherited),
        }
    }
}

impl Parse for ast::FunctionModifiers {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        let mut m = Self::default();

        if just(Token::KwConst).optional().parse_with(input)?.is_some() {
            m.const_ = true;
        }

        Ok(m)
    }
}

impl Parse for ast::Function {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        try {
            Self {
                attributes: attributes(false)(input)?,
                vis: ast::Visibility::parse(input)?,
                modifiers: ast::FunctionModifiers::parse(input)?,
                args: repeated_until(Token::OpenParen, ast::Type::parse, Token::CloseParen)(input)?,
                returns: ast::Type::parse
                    .optional()
                    .parse_with(input)?
                    .unwrap_or(ast::Type {
                        kind: ast::TypeKind::Primitive(ast::TypePrimitive::Void),
                        arguments: None,
                    }),
                name: ident(input)?,
                cap_args: ast::Pattern::parse.repeated().collect().parse_with(input)?,
                statements: {
                    let before = input.offset;

                    match input.next()? {
                        Token::Eq => vec![ast::Expr::parse(input)?],
                        Token::OpenCurly => {
                            let mut stmts = vec![];

                            while !matches!(input.peek()?, Token::CloseCurly) {
                                stmts.push(ast::Expr::parse(input)?);
                            }

                            input.skip()?;

                            stmts
                        }
                        other_token => {
                            return Err(ParserError::Expected {
                                expectation: vec![Expectation::AnyOf(vec![
                                    TokenKind::Eq,
                                    TokenKind::OpenCurly,
                                ])],
                                found: other_token,
                                at: input.span_since(before).into(),
                            });
                        }
                    }
                },
            }
        }
    }
}

impl Parse for ast::Expr {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        choice((
            just([Token::OpenParen, Token::CloseParen]).to(Self::Void),
            (
                (just(Token::KwLet), just(Token::KwMut))
                    .ignored()
                    .optional(),
                ast::Pattern::parse,
                just(Token::Colon).ignore_then(ast::Type::parse).optional(),
                just(Token::Eq).ignore_then(ast::Expr::parse),
            )
                .map(|(mutable, pattern, ty, value)| {
                    Self::Let(ast::LetExpr {
                        mutable: mutable.is_some(),
                        pattern,
                        ty,
                        value: Box::new(value),
                    })
                }),
            just(Token::KwTrue).to(Self::Primitive(ast::PrimitiveExpr::Bool(true))),
            just(Token::KwFalse).to(Self::Primitive(ast::PrimitiveExpr::Bool(false))),
            ident.map(Self::Opaque),
        ))
        .parse_with(input)
    }
}

impl Parse for ast::Pattern {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        choice((
            just(Token::KwRef)
                .ignore_then(Self::parse.map(Box::new))
                .map(Self::Ref),
            just(Token::KwMut)
                .ignore_then(Self::parse.map(Box::new))
                .map(Self::Mut),
            (just(Token::KwRef), just(Token::KwMut))
                .ignore_then(Self::parse.map(Box::new))
                .map(Self::RefMut),
            ident
                .then_ignore(just(Token::At))
                .then(Self::parse)
                .map(|(var, pat)| Self::WithVariable(var, Box::new(pat))),
            just([Token::OpenParen, Token::CloseParen]).to(Self::Void),
            ident.map(Self::Variable),
        ))
        .parse_with(input)
    }
}

impl Parse for ast::Type {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        use ast::{Type as T, TypeKind as K, TypeNumber as N, TypePrimitive as P};

        choice((
            just(Token::Excl).to(T {
                kind: K::Primitive(P::Never),
                arguments: None,
            }),
            just([Token::OpenParen, Token::CloseParen]).to(T {
                kind: K::Primitive(P::Void),
                arguments: None,
            }),
            just(Token::Amp)
                .ignore_then((
                    just(Token::Tick).ignore_then(ident).optional(),
                    just(Token::KwMut).optional(),
                    Self::parse,
                ))
                .map(|(lifetime, mutable, ty)| T {
                    kind: K::Reference(Box::new(TypeReference {
                        lifetime,
                        mutable: mutable.is_some(),
                        ty,
                    })),
                    arguments: None,
                }),
            ident
                .try_map(|pr, extra| {
                    Ok(match pr.as_str() {
                        "str" => P::Str,
                        "bool" => P::Bool,
                        "char" => P::Char,
                        int if int.starts_with("i") || int.starts_with("u") => P::Number(N::Int {
                            signed: int.starts_with("i"),
                            bits: int[1..].parse().map_err(|error| ParserError::NumberError {
                                error,
                                at: extra.span(),
                            })?,
                        }),
                        _ => {
                            return Err(ParserError::Expected {
                                expectation: vec![Expectation::Type],
                                found: Token::Identifier(pr),
                                at: extra.span(),
                            })
                        }
                    })
                })
                .map(|prim| T {
                    kind: K::Primitive(prim),
                    arguments: None,
                }),
        ))
        .parse_with(input)
    }
}

impl Parse for ast::Item {
    #[parser(extras = Extra)]
    fn parse(input: TokenStream) -> Self {
        choice((ast::Function::parse.map(ast::Item::Fn),)).parse_with(input)
    }
}

fn repeated_until<O>(
    bef: Token,
    parser: impl Parser<TokenStream, O, Extra>,
    until: Token,
) -> pfn_type!(TokenStream, Vec<O>, Extra) {
    move |input| try {
        just(bef.clone())(input)?;
        let mut real = vec![];

        while !matches!(input.peek()?, unt if unt == until) {
            real.push(parser.parse_with(input)?);
        }

        input.skip()?; // hit the until token

        real
    }
}

#[parser(extras = Extra)]
fn items(input: TokenStream) -> Vec<ast::Item> {
    let mut items = vec![];

    while input.peek().is_ok() {
        items.push(ast::Item::parse(input)?);
    }

    Ok(items)
}

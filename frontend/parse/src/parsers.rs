use aott::{extra, prelude::*, select};
use std::{borrow::Cow, mem::replace};
use tracing::*;

use super::{Error, *};
use aott::text::*;
use logos::Span;
use tangic_common::{ast::*, interner::IdentifierID};

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn numtype(input: &str) -> TypeNumber {
        let signed = match one_of(['i', 'u'])(input)? {
                'i' => Signedness::Signed,
                'u' => Signedness::Unsigned,
                _ => unsafe { std::hint::unreachable_unchecked() },
        };

        let before = input.offset;
        let bits = (&input.input[input.offset..std::cmp::min(input.offset + 3, input.input.len())])
                .parse::<u16>()
                .map_err(|e| Error {
                        span: input.span_since(before),
                        error: e.into(),
                })?;

        Ok(TypeNumber { signed, bits })
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn ident(input: Tokens) -> String {
        select!(Token::Ident(ident) => ident).parse_with(input)
}
#[parser(extras = Extra)]
#[instrument(ret, err)]
fn ident_id(input: Tokens) -> IdentifierID {
        select!(Token::Ident(ident) => IdentifierID::new(&ident)).parse_with(input)
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn path(input: Tokens) -> Path {
        enum State {
                Start(String),
                Delimiter,
                Done,
        }
        let mut state = State::Start(ident(input)?);
        let mut path = Path(vec![]);

        loop {
                match replace(&mut state, State::Done) {
                        State::Start(ident) => {
                                state = State::Delimiter;
                                path.0.push(IdentifierID::new(&ident))
                        }
                        State::Delimiter => {
                                state = match just(Token::Dot).optional().parse_with(input)? {
                                        Some(_) => State::Start(ident(input)?),
                                        None => State::Done,
                                };
                        }
                        State::Done => break,
                }
        }

        Ok(path)
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn typ(input: Tokens) -> Type {
        let before_ty = input.save();
        let t = match input.next()? {
                Token::Bang => Type::ErrorUnion(TypeErrorUnion {
                        error_type: None,
                        ok_type: Box::new(typ(input)?),
                }),
                Token::Ident(ident) => match ident.as_str() {
                        "bool" => Type::Primitive(TypePrimitive::Bool),
                        "never" => Type::Primitive(TypePrimitive::Never),
                        "str" => Type::Primitive(TypePrimitive::Str),

                        _ => {
                                if let Some(Token::Dot) = input.peek() {
                                        input.rewind(before_ty);

                                        Type::Path(path(input)?)
                                } else {
                                        Type::Path(Path(vec![IdentifierID::new(&ident)]))
                                }
                        }
                },
                Token::Ampersand => {
                        let mut t = TypeReference {
                                lifetime: None,
                                mutable: false,
                                inner: Box::new(Type::Primitive(TypePrimitive::Never)),
                        };
                        let before = input.save();

                        match input.next()? {
                                Token::SingleQuote => {
                                        t.lifetime = Some(ident_id(input)?);

                                        let before_ = input.save();
                                        if let Token::KeywordMut = input.next()? {
                                                t.mutable = true;
                                        } else {
                                                input.rewind(before_);
                                        }
                                }
                                Token::KeywordMut => t.mutable = true,

                                _ => t.inner = Box::new(typ(input)?),
                        };

                        Type::Reference(t)
                }
                Token::Question => Type::Nullable(Box::new(typ(input)?)),
                token => {
                        return Err(crate::Error {
                                span: input.span_since(before_ty.offset),
                                error: ParseError::Expected {
                                        expected: Cow::Borrowed("a type"),
                                        found: vec![token],
                                },
                        })
                }
        };

        Ok(if let Some(Token::Bang) = input.peek() {
                input.skip()?;

                Type::ErrorUnion(TypeErrorUnion {
                        error_type: Some(Box::new(t)),
                        ok_type: Box::new(typ(input)?),
                })
        } else {
                t
        })
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn expr(input: Tokens) -> Expression {
        Ok(match input.next()? {
                Token::KeywordTrue => Expression::Boolean(true),
                Token::KeywordFalse => Expression::Boolean(false),
                Token::Number(number) => Expression::Number(number),
                token => {
                        return Err(Error {
                                span: input.offset..input.offset,
                                error: ParseError::Expected {
                                        expected: Cow::Borrowed("an expression"),
                                        found: vec![token],
                                },
                        })
                }
        })
}

#[parser(extras = Extra)]
fn stmt_if(input: Tokens) -> ExprIf {
        enum Style {
                Quick,
                C,
                Rust,
        }
        try {
                let style = Style::Quick;
                let condition = match input.peek() {
                        Token::OpenParen => {
                                style = Style::C;

                                delimited(just(Token::OpenParen), expr, just(Token::CloseParen))(
                                        input,
                                )?
                        }

                        _ => expr(input)?,
                };
                match input.peek() {
                        Token::FatArrow => {}
                }
                ExprIf {}
        }
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn stmt(input: Tokens) -> Statement {
        try {
                let before = input.save();
                match input.next()? {
                        Token::KeywordReturn => Statement::Return(ExprReturn {
                                is_implicit: false,
                                value: Box::new(
                                        expr.then_ignore(just(Token::Semi).optional())
                                                .parse_with(input)?,
                                ),
                        }),
                        Token::KeywordIf => {
                                input.rewind(before);
                                Statement::If(stmt_if(input)?)
                        }
                        token => {
                                return Err(Error {
                                        span: input.offset..input.offset,
                                        error: ParseError::Expected {
                                                expected: Cow::Borrowed("a statement"),
                                                found: vec![token],
                                        },
                                })
                        }
                }
        }
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn item_fn(input: Tokens) -> ItemFn {
        try {
                just(Token::KeywordFn)(input)?;

                // Nested parsers.
                // Nested parsers.
                #[parser(extras = Extra)]
                fn arg(input: Tokens) -> FnArgument {
                        try {
                                FnArgument {
                                        name: ident_id
                                                .then_ignore(just(Token::Colon))
                                                .parse_with(input)?,
                                        ty: typ(input)?,
                                }
                        }
                }

                ItemFn {
                        ident: ident_id(input)?,
                        args: delimited(
                                just(Token::OpenParen),
                                arg.separated_by(just(Token::Comma)).allow_trailing(),
                                just(Token::CloseParen),
                        )(input)?,
                        return_type: typ(input)?,
                        block: Block {
                                label: None,
                                statements: delimited(
                                        just(Token::OpenCurly),
                                        stmt.repeated(),
                                        just(Token::CloseCurly),
                                )(input)?,
                        },
                }
        }
}
// #[parser(extras = Extra)]
// fn item_const(input: Tokens) -> ItemConst {
//         just(Token::KeywordConst)(input)?;

//         let ident = ident_id(input)?;

//         just(Token::Colon)(input)?;

//         let ty = typ(input)?;

//         just(Token::PunctEq)(input)?;

//         let value = expr(input)?;

//         Ok(ItemConst { ident, ty, value })
// }

#[parser(extras = Extra)]
#[instrument(ret, err)]
fn item(input: Tokens) -> Item {
        Ok(match input.peek() {
                Some(Token::KeywordFn) => item_fn(input).map(Item::Function)?,
                // Some(Token::KeywordConst) => item_const(input).map(Item::Constant),
                Some(token) => {
                        return Err(Error {
                                span: input.offset..(input.offset + 1),
                                error: ParseError::Expected {
                                        expected: Cow::Borrowed(
                                                "an item (a function, a constant etc.)",
                                        ),
                                        found: vec![token],
                                },
                        })
                }
                None => {
                        return Err(Error {
                                span: input.offset..(input.offset + 1),
                                error: ParseError::UnexpectedEof,
                        })
                }
        })
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
pub fn file(input: Tokens) -> File {
        try {
                File {
                        items: item.repeated().then_ignore(end).parse_with(input)?,
                }
        }
}

#[parser(extras = Extra)]
#[instrument(ret, err)]
pub fn parse_number(input: &str) -> ExprNumber {
        const HEX: u32 = 16;
        const BIN: u32 = 2;
        const OCT: u32 = 8;

        let radix = choice((just("0x").to(HEX), just("0b").to(BIN), just("0o").to(OCT)))
                .optional()
                .map(|radix| radix.unwrap_or(10))
                .parse_with(input)?;
        let before = input.offset;
        let digits = slice(digits(radix)).parse_with(input)?;
        let ty = numtype.optional().parse_with(input)?;
        Ok(ExprNumber::Normal(ExprNumberNormal {
                number: digits.parse::<i64>().map_err(|e| Error {
                        span: input.span_since(before),
                        error: e.into(),
                })?,
                radix,
                ty,
        }))
}

use aott::{extra, prelude::*, select};
use std::borrow::Cow;
use tracing::*;

use super::{Error, *};
use aott::text::*;
use logos::Span;
use tangic_common::{
        ast::{
                self, Block, ExprCall, ExprNumber, ExprNumberNormal, ExprReturn, Expression, File,
                Item, ItemFn, Path, Signedness, Statement, Type, TypeArray, TypeErrorUnion,
                TypeNumber, TypePrimitive,
        },
        interner::IdentifierID,
};

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn nrtype_ident(input: &str) -> TypeNumber {
        let signed = choice((
                just('u').to(Signedness::Unsigned),
                just('i').to(Signedness::Signed),
        ))
        .parse_with(input)?;
        let bits = digits(10)
                .slice()
                .try_map_with_span(|s: &str, span| {
                        s.parse::<u16>().map_err(|e| Error {
                                span,
                                error: e.into(),
                        })
                })
                .parse_with(input)?;
        Ok(TypeNumber { signed, bits })
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn ident(input: Tokens) -> String {
        select!(Token::Ident(id) => id).parse_with(input)
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn usize_token(input: Tokens) -> usize {
        select!(Token::Number(ExprNumber::Normal(ExprNumberNormal { radix: 10, number, .. })) => number as usize).parse_with(input)
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn typ(input: Tokens) -> Type {
        let number_type = ident.try_map(|s: String| nrtype_ident.parse(s.as_str()));
        let array_type = just(Token::OpenBracket)
                .ignore_then(usize_token.optional())
                .then(typ)
                .then_ignore(just(Token::CloseBracket))
                .map(|(size, inner)| TypeArray {
                        inner: Box::new(inner),
                        size,
                });
        let typ_primitive = choice((
                number_type.map(TypePrimitive::Number),
                array_type.map(TypePrimitive::Array),
                select! {
                        Token::Ident(id) if id == "void" => TypePrimitive::Void,
                        Token::Ident(id) if id == "str" => TypePrimitive::Str,
                        Token::Ident(id) if id == "never" => TypePrimitive::Never
                },
        ));
        let typ_nullable = just(Token::Question).ignore_then(typ);
        let typ_error_union = typ.optional().then(just(Token::Bang).ignore_then(typ)).map(
                |(error_type, ok_type)| TypeErrorUnion {
                        error_type: Box::new(error_type),
                        ok_type: Box::new(ok_type),
                },
        );
        choice((
                typ_primitive.map(Type::Primitive),
                typ_nullable.map(Box::new).map(Type::Nullable),
                typ_error_union.boxed().map(Type::ErrorUnion),
        ))
        .parse_with(input)
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn ident_id(input: Tokens) -> IdentifierID {
        Ok(IdentifierID::new(&ident(input)?))
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn path(input: Tokens) -> Path {
        ident_id.separated_by(just(Token::Dot))
                .map(Path)
                .parse_with(input)
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn expr(input: Tokens) -> Expression {
        let call = path
                .then_ignore(just(Token::OpenParen))
                .then_ignore(just(Token::CloseParen))
                .map(|callee| ExprCall {
                        callee,
                        args: vec![],
                });
        let retn = just(Token::KeywordReturn)
                .ignore_then(expr)
                .then_ignore(just(Token::Semi))
                .map(|expr| ExprReturn {
                        is_implicit: false,
                        value: Box::new(expr),
                })
                .or(expr.map(|expr| ExprReturn {
                        is_implicit: true,
                        value: Box::new(expr),
                }));
        let number = select!(Token::Number(number) => number);
        choice((
                number.map(Expression::Number),
                retn.map(Expression::Return),
                call.map(Expression::Call),
                path.map(Expression::Access),
        ))
        .parse_with(input)
}

fn expected_statement(span: Span) -> Error {
        Error {
                span,
                error: ParseError::Other(Cow::Borrowed("expected statement")),
        }
}

macro_rules! try_match {
        ($initial:expr; $expected:ident =>> {$($pat:pat$(if $guard:expr)? => $expr:expr),*}) => {
                $initial.try_map_with_span(|__out, __span| match __out {
                        $($pat$(if $guard)? => Ok($expr)),*,
                        _ => Err($expected(__span))
                })
        };
        ($initial:expr; $expected:ident =>> $ty:ident$(::$en:ident)?($field:ident)$(if $guard:expr)? => $expr:expr) => {
                try_match!($initial; $expected =>> { $ty$(::$en)?($field)$(if $guard)? => $expr })
        };
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn stmt(input: Tokens) -> Statement {
        let before = input.offset;
        Ok(match expr(input)? {
                Expression::Call(call) => Statement::Call(call),
                Expression::Return(ret) => Statement::Return(ret),
                _ => return Err(expected_statement(input.span_since(before))),
        })
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
fn block(input: Tokens) -> Block {
        delimited(
                just(Token::OpenCurly),
                stmt.repeated(),
                just(Token::CloseCurly),
        )
        .map(|statements| Block {
                label: None,
                statements,
        })
        .parse_with(input)
}

#[parser(extras = Extra)]
#[tracing::instrument(ret)]
fn item_fn(input: Tokens) -> ItemFn {
        let fn_arg = ident_id
                .then_ignore(just(Token::Colon))
                .then(typ)
                .map(|(name, ty)| ast::FnArgument { name, ty });

        just(Token::KeywordFn).check_with(input)?;
        Ok(ItemFn {
                ident: ident_id(input)?,
                args: delimited(
                        just(Token::OpenParen),
                        fn_arg.separated_by(just(Token::Comma)).allow_trailing(),
                        just(Token::CloseParen),
                )(input)?,
                return_type: typ(input)?,
                block: block(input)?,
        })
}

#[parser(extras = "Extra")]
#[tracing::instrument(ret)]
pub fn file(input: Tokens) -> File {
        item_fn.map(Item::Function)
                .repeated()
                .map(|items| ast::File { items })
                .then_ignore(end)
                .parse_with(input)
}

#[parser(extras = "Extra")]
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
        let ty = nrtype_ident.optional().parse_with(input)?;
        Ok(ExprNumber::Normal(ExprNumberNormal {
                number: digits.parse::<i64>().map_err(|e| Error {
                        span: input.span_since(before),
                        error: e.into(),
                })?,
                radix,
                ty,
        }))
}

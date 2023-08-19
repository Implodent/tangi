use chumsky::{prelude::*, text::digits};
use tangic_common::{
        ast::{
                self, ItemFn, Signedness, Type, TypeArray, TypeErrorUnion, TypeNumber,
                TypePrimitive,
        },
        interner::IdentifierID,
};

use crate::{Error, Token};

fn nrtype_ident() -> impl Parser<char, TypeNumber, Error = Error> {
        choice((
                just('u').to(Signedness::Unsigned),
                just('i').to(Signedness::Signed),
        ))
        .then(digits::<char, Error>(10).try_map(|s: String, span| {
                s.parse::<u16>().map_err(|e| Error::new(span, e.into()))
        }))
        .map(|(signed, bits)| TypeNumber { signed, bits })
}

fn ident() -> impl Parser<Token, String, Error = Error> {
        select!(Token::Ident(id) => id)
}

fn usize_token() -> impl Parser<Token, usize, Error = Error> {
        select!(Token::Usize(u) => u)
}

fn typ() -> impl Parser<Token, ast::Type, Error = Error> {
        recursive(|typ| {
                let number_type = ident().try_map(|str, _| {
                        nrtype_ident().parse(str).map_err(|e| {
                                e.into_iter().fold(Error::default(), |err, next_error| {
                                        err.merge_(next_error)
                                })
                        })
                });
                let array_type = just(Token::OpenBracket)
                        .ignore_then(usize_token().or_not())
                        .then(typ.clone())
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
                let typ_nullable = just(Token::Question).ignore_then(typ.clone());
                let typ_error_union = just(Token::Bang)
                        .ignore_then(typ.clone())
                        .then(typ.or_not())
                        .map(|(ok_type, error_type)| TypeErrorUnion {
                                error_type: Box::new(error_type),
                                ok_type: Box::new(ok_type),
                        });
                choice((
                        typ_primitive.map(Type::Primitive),
                        typ_nullable.map(Box::new).map(Type::Nullable),
                        typ_error_union.map(Type::ErrorUnion),
                ))
        })
}

fn ident_id() -> impl Parser<Token, IdentifierID, Error = Error> {
        ident().map(|id| IdentifierID::new(&id))
}

fn block() -> impl Parser<Token, Block, Error = Error> {
        stmt().separated_by(just(Token::Semi))
}

pub fn help() -> impl Parser<Token, ast::File, Error = Error> {
        let fn_arg = ident_id()
                .then_ignore(just(Token::Colon))
                .then(typ())
                .map(|(name, ty)| ast::FnArgument { name, ty });
        let item_fn = just(Token::KeywordFn)
                .ignore_then(ident_id())
                .then_ignore(just(Token::OpenParen))
                .then(fn_arg.separated_by(just(Token::Comma)).allow_trailing())
                .then_ignore(just(Token::CloseParen))
                .then(typ())
                .then(block())
                .map(|(((ident, args), return_type), block)| ItemFn {
                        ident,
                        args,
                        return_type,
                });
        let item = choice((item_fn.map(ast::Item::Function),));
        item.repeated()
                .map(|items| ast::File { items })
                .then_ignore(end())
}

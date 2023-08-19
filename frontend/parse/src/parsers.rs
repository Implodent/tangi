use chumsky::{error::Cheap, prelude::*, text::digits};
use tangic_common::{
        ast::{self, ItemFn, Signedness, Type, TypeNumber, TypePrimitive},
        interner::IdentifierID,
};

use crate::{Error, Token};

fn nrtype_ident() -> impl Parser<char, TypeNumber, Error = Error> {
        choice((
                just('u').to(Signedness::Unsigned),
                just('i').to(Signedness::Signed),
        ))
        .then(digits::<char, Error>(10).try_map(|s: String, span| {
                (&s[0..3])
                        .parse::<u16>()
                        .map_err(|e| Error::new(span, e.into()))
        }))
        .map(|(signed, bits)| TypeNumber { signed, bits })
}

pub fn help() -> impl Parser<Token, ast::File, Error = Error> {
        let ident = select!(Token::Ident(id) => id);
        let ident_id = ident.clone().map(|id| IdentifierID::new(&id));
        let number_type = ident.clone().try_map(|str, _| {
                nrtype_ident().parse(str).map_err(|e| {
                        e.into_iter()
                                .fold(Error::default(), |err, next_error| err.merge_(next_error))
                })
        });
        let typ_primitive = choice((number_type.map(TypePrimitive::Number),));
        let typ = choice((typ_primitive.map(Type::Primitive),));
        let fn_arg = ident_id
                .clone()
                .then_ignore(just(Token::Colon))
                .then(typ)
                .map(|(name, ty)| ast::FnArgument { name, ty });
        let item_fn = just(Token::KeywordFn)
                .ignore_then(ident_id)
                .then_ignore(just(Token::OpenParen))
                .then(fn_arg.separated_by(just(Token::Comma)).allow_trailing())
                .then_ignore(just(Token::CloseParen))
                .then(typ)
                .map(|((ident, args), return_type)| ItemFn {
                        ident,
                        args,
                        return_type,
                });
        let item = choice((item_fn.map(ast::Item::Function),));
        item.repeated().map(|items| ast::File { items })
}

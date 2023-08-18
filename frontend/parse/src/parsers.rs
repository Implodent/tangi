use chumsky::{error::Cheap, prelude::*, text::digits};
use tangic_common::{
        ast::{self, Signedness, TypeNumber},
        interner::IdentifierID,
};

use crate::{Error, Token};

pub fn help() -> impl Parser<Token, ast::File, Error = Error> {
        let ident = select!(Token::Ident(id) => id).map(|id| (id, IdentifierID::new(&id)));
        let nrtype_ident = choice((
                just('u').to(Signedness::Unsigned),
                just('i').to(Signedness::Signed),
        ))
        .then(digits::<char, Cheap<char>>(10).try_map(|s: String, _| &s[0..3].parse::<u16>()))
        .map(|(signed, bits)| TypeNumber { signed, bits });
        let number_type = ident
                .map(|(id, _)| id)
                .try_map(|str, _| nrtype_ident.parse(&str));
        let typ = choice((number_type,));
        let fn_arg = ident
                .then_ignore(just(Token::Colon))
                .then(typ)
                .map(|(name, ty)| ast::FnArgument { name, ty });
        let item_fn = just(Token::KeywordFn)
                .ignore_then(ident)
                .then_ignore(just(Token::OpenParen))
                .then(fn_arg.repeated());
        let item = choice((item_fn.map(ast::Item::Function),));
        item.repeated().map(|items| ast::File { items })
}

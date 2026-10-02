use super::{Err, TokenInput, block, comma_list, ident, stmt, ty};
use crate::{Field, FnDef, Item, Param, StructDef, Token};
use chumsky::prelude::*;

pub fn item<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    let module = just(Token::Mod)
        .ignore_then(ident())
        .then_ignore(just(Token::Semi))
        .map(Item::Mod);
    let use_ = just(Token::Use)
        .ignore_then(
            ident()
                .separated_by(just(Token::ColonColon))
                .at_least(1)
                .collect(),
        )
        .then_ignore(just(Token::Semi))
        .map(Item::Use);

    let field = ident()
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map(|(name, ty)| Field { name, ty });
    let struct_def = just(Token::Rev)
        .ignore_then(just(Token::Struct))
        .ignore_then(ident())
        .then(comma_list(field).delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map(|(name, fields)| Item::Struct(StructDef { name, fields }));

    let param = ident()
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map(|(name, ty)| Param { name, ty });
    let fn_def = just(Token::Rev)
        .ignore_then(just(Token::Fn))
        .ignore_then(ident())
        .then(comma_list(param).delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block(stmt()))
        .map(|((name, params), body)| Item::Fn(FnDef { name, params, body }));

    module.or(use_).or(struct_def).or(fn_def)
}

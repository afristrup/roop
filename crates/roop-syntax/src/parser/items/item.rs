use crate::{Err, TokenInput, block, comma_list, enum_def, ident, param, stmt, struct_def};
use crate::{FnDef, Item, Token, UseDecl};
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
        .then(just(Token::As).ignore_then(ident()).or_not())
        .then_ignore(just(Token::Semi))
        .map(|(path, alias)| Item::Use(UseDecl { path, alias }));
    let fn_def = just(Token::Pub)
        .or_not()
        .then(just(Token::Irrev).or_not())
        .then_ignore(just(Token::Fn))
        .then(ident())
        .then(comma_list(param()).delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block(stmt()))
        .map(|((((public, irrev), name), params), body)| {
            Item::Fn(FnDef {
                name,
                params,
                body,
                irreversible: irrev.is_some(),
                public: public.is_some(),
            })
        });

    module
        .or(use_)
        .or(enum_def().map(Item::Enum))
        .or(struct_def().map(Item::Struct))
        .or(fn_def)
}

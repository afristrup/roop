use crate::{
    Err, TokenInput, bennett_def, block, comma_list, enum_def, generics, ident, param, session_def,
    stmt, struct_def, test_def, use_decl,
};
use crate::{FnDef, Item, Token};
use chumsky::prelude::*;

pub fn item<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    let module = just(Token::Mod)
        .ignore_then(ident())
        .then_ignore(just(Token::Semi))
        .map(Item::Mod);
    let fn_def = just(Token::Pub)
        .or_not()
        .then(just(Token::Irrev).or_not())
        .then_ignore(just(Token::Fn))
        .then(ident())
        .then(generics().or_not())
        .then(comma_list(param()).delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block(stmt()))
        .map(|(((((public, irrev), name), generics), params), body)| {
            Item::Fn(FnDef {
                name,
                generics: generics.unwrap_or_default(),
                params,
                body,
                irreversible: irrev.is_some(),
                public: public.is_some(),
                test: false,
                bennett: None,
            })
        });

    module
        .or(test_def())
        .or(bennett_def())
        .or(use_decl().map(Item::Use))
        .or(session_def().map(Item::Session))
        .or(enum_def().map(Item::Enum))
        .or(struct_def().map(Item::Struct))
        .or(fn_def)
}

use crate::{Block, Einsum, FnDef, Item, Token, unescape};
use crate::{Err, TokenInput, ident, ty};
use chumsky::prelude::*;

/// `[pub] einsum fn name: T = "ij,jk->ik";`: a function with no signature of
/// its own, which `roop-opt` writes from the subscripts.
pub fn einsum_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    let spec = select! { Token::Str(s) => s }.try_map(|s, span| {
        unescape(&s[1..s.len() - 1])
            .map_err(|e| Rich::custom(span, e))
            .and_then(|b| String::from_utf8(b).map_err(|e| Rich::custom(span, e.to_string())))
    });
    just(Token::Pub)
        .or_not()
        .then_ignore(select! { Token::Ident("einsum") => () })
        .then_ignore(just(Token::Fn))
        .then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .then_ignore(just(Token::Assign))
        .then(spec)
        .then_ignore(just(Token::Semi))
        .map(|(((public, name), elem), spec)| {
            Item::Fn(FnDef {
                name,
                generics: Vec::new(),
                params: Vec::new(),
                body: Block::default(),
                irreversible: false,
                public: public.is_some(),
                test: false,
                bennett: None,
                external: false,
                world: false,
                einsum: Some(Einsum { spec, elem }),
            })
        })
}

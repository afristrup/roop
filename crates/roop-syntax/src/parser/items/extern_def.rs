use crate::{Block, FnDef, Item, Token};
use crate::{Err, TokenInput, comma_list, generics, ident, param};
use chumsky::prelude::*;

/// `[pub] extern fn name<N>(params);`: a function the runtime provides, which
/// takes its parameters by reference like any other. With `world`, as in
/// `extern world fn`, it is reversible: the runtime provides `name_inv` as well,
/// and the results it writes are into places that start zero.
pub fn extern_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    just(Token::Pub)
        .or_not()
        .then_ignore(just(Token::Extern))
        .then(select! { Token::Ident("world") => () }.or_not())
        .then_ignore(just(Token::Fn))
        .then(ident())
        .then(generics().or_not())
        .then(comma_list(param()).delimited_by(just(Token::LParen), just(Token::RParen)))
        .then_ignore(just(Token::Semi))
        .map(|((((public, world), name), generics), params)| {
            Item::Fn(FnDef {
                name,
                generics: generics.unwrap_or_default(),
                params,
                body: Block::default(),
                irreversible: world.is_none(),
                public: public.is_some(),
                test: false,
                bennett: None,
                external: true,
                world: world.is_some(),
                einsum: None,
            })
        })
}

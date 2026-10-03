use crate::{Block, FnDef, Item, Token};
use crate::{Err, TokenInput, ident};
use chumsky::prelude::*;

/// `[pub] bennett fn name = target;`: a function with no signature of its own,
/// which `roop-opt` fills in from `target`.
pub fn bennett_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Item, Err<'a>> + Clone {
    just(Token::Pub)
        .or_not()
        .then_ignore(select! { Token::Ident("bennett") => () })
        .then_ignore(just(Token::Fn))
        .then(ident())
        .then_ignore(just(Token::Assign))
        .then(ident())
        .then_ignore(just(Token::Semi))
        .map(|((public, name), target)| {
            Item::Fn(FnDef {
                name,
                generics: Vec::new(),
                params: Vec::new(),
                body: Block::default(),
                irreversible: false,
                public: public.is_some(),
                test: false,
                bennett: Some(target),
            })
        })
}

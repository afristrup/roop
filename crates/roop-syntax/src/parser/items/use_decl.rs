use crate::{Err, Token, TokenInput, UseDecl, UseShape, comma_list, ident};
use chumsky::prelude::*;

/// `[pub] use path;`, `use path as name;`, `use path::*;` or `use path::{a, b as c};`
pub fn use_decl<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, UseDecl, Err<'a>> + Clone {
    let renamed = ident().then(just(Token::As).ignore_then(ident()).or_not());
    let group = comma_list(renamed).delimited_by(just(Token::LBrace), just(Token::RBrace));
    let tail = just(Token::Star)
        .to(UseShape::Glob)
        .or(group.map(UseShape::Group));
    just(Token::Pub)
        .or_not()
        .then_ignore(just(Token::Use))
        .then(
            ident()
                .separated_by(just(Token::ColonColon))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then(just(Token::ColonColon).ignore_then(tail).or_not())
        .then(just(Token::As).ignore_then(ident()).or_not())
        .then_ignore(just(Token::Semi))
        .map(|(((public, path), tail), alias)| UseDecl {
            path,
            alias,
            public: public.is_some(),
            shape: tail.unwrap_or(UseShape::Single),
        })
}

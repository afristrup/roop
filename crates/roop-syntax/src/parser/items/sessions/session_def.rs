use crate::{Err, Role, SessionDef, Token, TokenInput, behaviour, ident};
use chumsky::prelude::*;

/// `[pub] session Name { role: behaviour; ... }`
pub fn session_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, SessionDef, Err<'a>> + Clone {
    let role = ident()
        .then_ignore(just(Token::Colon))
        .then(behaviour())
        .then_ignore(just(Token::Semi))
        .map(|(name, behaviour)| Role { name, behaviour });
    just(Token::Pub)
        .or_not()
        .then_ignore(just(Token::Session))
        .then(ident())
        .then(
            role.repeated()
                .at_least(1)
                .collect()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map_with(|((public, name), roles), e| SessionDef {
            name,
            roles,
            span: e.span(),
            public: public.is_some(),
        })
}

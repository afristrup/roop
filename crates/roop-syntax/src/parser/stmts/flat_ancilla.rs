use crate::{Entry, Err, Token, TokenInput, expr, ident, ty};
use chumsky::prelude::*;

/// `ancilla name: T = e;`, an ancilla that lasts to the end of its block.
pub fn flat_ancilla<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Entry, Err<'a>> + Clone {
    just(Token::Auto)
        .or_not()
        .then_ignore(just(Token::Ancilla))
        .then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .then_ignore(just(Token::Assign))
        .then(expr())
        .then_ignore(just(Token::Semi))
        .map_with(|(((auto, name), ty), init), e| {
            let span: crate::Span = e.span();
            Entry::Declare {
                name,
                ty,
                init,
                auto: auto.is_some(),
                start: span.start,
                after: span.end,
            }
        })
}

use super::{Err, TokenInput};
use crate::{Pattern, Token};
use chumsky::prelude::*;

pub fn pattern<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Pattern, Err<'a>> + Clone {
    let int = select! { Token::Int(s) => s }.try_map(|s, span| {
        s.parse::<i64>()
            .map(Pattern::Int)
            .map_err(|e| Rich::custom(span, e.to_string()))
    });
    let boolean =
        select! { Token::True => Pattern::Bool(true), Token::False => Pattern::Bool(false) };
    let wildcard = just(Token::Underscore).to(Pattern::Wildcard);
    int.or(boolean).or(wildcard)
}

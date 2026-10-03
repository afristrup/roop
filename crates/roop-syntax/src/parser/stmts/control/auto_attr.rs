use crate::{Attr, Err, Token, TokenInput, lifetime};
use chumsky::prelude::*;

/// `auto` or `auto<'region>`, before an ancilla.
pub fn auto_attr<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Attr, Err<'a>> + Clone {
    just(Token::Auto)
        .ignore_then(
            lifetime()
                .delimited_by(just(Token::Lt), just(Token::Gt))
                .or_not(),
        )
        .map(|region| Attr::Auto { region })
}

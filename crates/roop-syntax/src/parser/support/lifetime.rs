use super::{Err, TokenInput};
use crate::Token;
use chumsky::prelude::*;

/// A lifetime name such as `'round`, without the quote.
pub fn lifetime<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, String, Err<'a>> + Clone {
    select! { Token::Lifetime(s) => s[1..].to_string() }
}

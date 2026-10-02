use super::{Err, TokenInput, ident};
use crate::Token;
use chumsky::prelude::*;

/// `Enum::Variant`, shared by expressions and patterns.
pub fn variant<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, (String, String), Err<'a>> + Clone {
    ident().then_ignore(just(Token::ColonColon)).then(ident())
}

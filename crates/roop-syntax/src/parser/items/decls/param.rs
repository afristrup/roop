use crate::{Err, TokenInput, ident, ty};
use crate::{Param, Token};
use chumsky::prelude::*;

pub fn param<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Param, Err<'a>> + Clone {
    ident()
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map(|(name, ty)| Param { name, ty })
}

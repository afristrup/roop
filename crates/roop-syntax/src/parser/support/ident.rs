use crate::Token;
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn ident<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, String, Err<'a>> + Clone {
    select! { Token::Ident(s) => s.to_string() }
}

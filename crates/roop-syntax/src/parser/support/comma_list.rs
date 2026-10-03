use crate::Token;
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn comma_list<'a, I: TokenInput<'a>, O>(
    p: impl Parser<'a, I, O, Err<'a>> + Clone,
) -> impl Parser<'a, I, Vec<O>, Err<'a>> + Clone {
    p.separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
}

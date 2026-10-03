use crate::{BuildFn, Token};
use crate::{Err, TokenInput, block, comma_list, param, stmt};
use chumsky::prelude::*;

pub fn build_fn<'a, I: TokenInput<'a>>(
    keyword: Token<'a>,
) -> impl Parser<'a, I, BuildFn, Err<'a>> + Clone {
    just(keyword)
        .ignore_then(comma_list(param()).delimited_by(just(Token::LParen), just(Token::RParen)))
        .then(block(stmt()))
        .map(|(params, body)| BuildFn { params, body })
}

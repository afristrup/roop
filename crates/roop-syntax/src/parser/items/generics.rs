use crate::{Err, Token, TokenInput, comma_list, ident};
use chumsky::prelude::*;

/// `<N, M>`: the length parameters of a function.
pub fn generics<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Vec<String>, Err<'a>> + Clone {
    comma_list(ident()).delimited_by(just(Token::Lt), just(Token::Gt))
}

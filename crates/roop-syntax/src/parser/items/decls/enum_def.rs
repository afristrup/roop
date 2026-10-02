use crate::{EnumDef, Token};
use crate::{Err, TokenInput, comma_list, ident};
use chumsky::prelude::*;

pub fn enum_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, EnumDef, Err<'a>> + Clone {
    just(Token::Enum)
        .ignore_then(ident())
        .then(comma_list(ident()).delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, variants), e| EnumDef {
            name,
            variants,
            span: e.span(),
        })
}

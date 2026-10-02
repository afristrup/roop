use crate::{Err, TokenInput, build_fn, comma_list, ident, ty};
use crate::{Field, StructDef, Token};
use chumsky::prelude::*;

pub fn struct_def<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, StructDef, Err<'a>> + Clone {
    let field = ident()
        .then_ignore(just(Token::Colon))
        .then(ty())
        .map(|(name, ty)| Field { name, ty });
    let members = comma_list(field)
        .then(build_fn(Token::Build).or_not())
        .then(build_fn(Token::Unbuild).or_not())
        .delimited_by(just(Token::LBrace), just(Token::RBrace));
    just(Token::Rev)
        .ignore_then(just(Token::Struct))
        .ignore_then(ident())
        .then(members)
        .map_with(|(name, ((fields, build), unbuild)), e| StructDef {
            name,
            fields,
            build,
            unbuild,
            span: e.span(),
        })
}

use crate::{Err, TokenInput, ident};
use crate::{Expr, Place, Token};
use chumsky::prelude::*;

enum Suffix {
    Field(String),
    Index(Expr),
}

pub fn place<'a, I: TokenInput<'a>>(
    expr: impl Parser<'a, I, Expr, Err<'a>> + Clone,
) -> impl Parser<'a, I, Place, Err<'a>> + Clone {
    let field = just(Token::Dot).ignore_then(ident()).map(Suffix::Field);
    let index = expr
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map(Suffix::Index);
    ident()
        .map(Place::Var)
        .foldl(field.or(index).repeated(), |base, suffix| match suffix {
            Suffix::Field(f) => Place::Field(Box::new(base), f),
            Suffix::Index(i) => Place::Index(Box::new(base), Box::new(i)),
        })
}

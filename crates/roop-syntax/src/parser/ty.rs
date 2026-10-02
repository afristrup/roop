use super::{Err, TokenInput, ident};
use crate::{Token, Type};
use chumsky::prelude::*;

pub fn ty<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Type, Err<'a>> + Clone {
    recursive(|ty| {
        let named = ident().map(Type::Named);
        let reference = just(Token::Amp)
            .ignore_then(just(Token::Mut).or_not())
            .then(ty.clone())
            .map(|(m, inner)| Type::Ref {
                mutable: m.is_some(),
                inner: Box::new(inner),
            });
        let len = select! { Token::Int(s) => s }.try_map(|s, span| {
            s.parse::<u64>()
                .map_err(|e| Rich::custom(span, e.to_string()))
        });
        let array = ty
            .then_ignore(just(Token::Semi))
            .then(len)
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|(t, n)| Type::Array(Box::new(t), n));
        named.or(reference).or(array)
    })
}

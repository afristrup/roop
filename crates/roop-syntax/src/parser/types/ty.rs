use crate::{Err, TokenInput, ident};
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
        let sized = len.map(Ok).or(ident().map(Err));
        let array = ty
            .clone()
            .then_ignore(just(Token::Semi))
            .then(sized.clone())
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|(t, n)| match n {
                Ok(n) => Type::Array(Box::new(t), n),
                Err(len) => Type::Param {
                    elem: Box::new(t),
                    len,
                    stack: false,
                },
            });
        let stack = select! { Token::Ident("Stack") => () }
            .ignore_then(
                ty.then_ignore(just(Token::Comma))
                    .then(sized)
                    .delimited_by(just(Token::Lt), just(Token::Gt)),
            )
            .map(|(t, n)| match n {
                Ok(n) => Type::Stack(Box::new(t), n),
                Err(len) => Type::Param {
                    elem: Box::new(t),
                    len,
                    stack: true,
                },
            });
        stack.or(named).or(reference).or(array)
    })
}

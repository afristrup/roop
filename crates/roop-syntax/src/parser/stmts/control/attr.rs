use crate::{Attr, Target, Token};
use crate::{Err, TokenInput, ident};
use chumsky::prelude::*;

/// `#[concurrent]`, `#[parallel]` or `#[parallel(cpu | cuda | metal)]`.
pub fn attr<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Attr, Err<'a>> + Clone {
    let named = |word: &'static str| {
        ident().try_map(move |n, span| {
            if n == word {
                Ok(())
            } else {
                Err(Rich::custom(span, format!("unknown attribute `{n}`")))
            }
        })
    };
    let target = ident().try_map(|n, span| match n.as_str() {
        "cpu" => Ok(Target::Cpu),
        "cuda" => Ok(Target::Cuda),
        "metal" => Ok(Target::Metal),
        _ => Err(Rich::custom(span, format!("unknown parallel target `{n}`"))),
    });
    let parallel = named("parallel")
        .ignore_then(
            target
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map(|target| Attr::Parallel { target });
    let concurrent = named("concurrent").to(Attr::Concurrent);
    just(Token::Hash).ignore_then(
        parallel
            .or(concurrent)
            .delimited_by(just(Token::LBracket), just(Token::RBracket)),
    )
}

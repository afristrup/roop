use crate::{Attr, Target, Token};
use crate::{Err, TokenInput, ident};
use chumsky::prelude::*;

/// `#[parallel]` or `#[parallel(cpu | nvptx | metal)]`.
pub fn attr<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Attr, Err<'a>> + Clone {
    let name = ident().try_map(|n, span| match n.as_str() {
        "parallel" => Ok(()),
        _ => Err(Rich::custom(span, format!("unknown attribute `{n}`"))),
    });
    let target = ident().try_map(|n, span| match n.as_str() {
        "cpu" => Ok(Target::Cpu),
        "nvptx" => Ok(Target::Nvptx),
        "metal" => Ok(Target::Metal),
        _ => Err(Rich::custom(span, format!("unknown parallel target `{n}`"))),
    });
    let concurrent = ident().try_map(|n, span| match n.as_str() {
        "concurrent" => Ok(Attr::Concurrent),
        _ => Err(Rich::custom(span, format!("unknown attribute `{n}`"))),
    });
    just(Token::Hash).ignore_then(
        concurrent.delimited_by(just(Token::LBracket), just(Token::RBracket)).or(name.ignore_then(
            target
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .or_not(),
        )
        .map(|target| Attr::Parallel { target })
        .delimited_by(just(Token::LBracket), just(Token::RBracket)),
    )
}

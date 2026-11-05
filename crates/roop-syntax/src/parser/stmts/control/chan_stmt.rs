use crate::{Block, StmtKind, Token, Type};
use crate::{Err, TokenInput, ident, ty};
use chumsky::prelude::*;

pub fn chan_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Chan)
        .ignore_then(ident())
        .then_ignore(just(Token::Colon))
        .then(ty())
        .then(
            ident()
                .try_map(|name, span| {
                    if name == "session" {
                        Ok(())
                    } else {
                        Err(Rich::custom(span, "expected `session`"))
                    }
                })
                .ignore_then(ident())
                .or_not(),
        )
        .then(blk)
        .map(
            |(((name, ty), session), body): (((String, Type), Option<String>), Block)| {
                StmtKind::Chan {
                    name,
                    ty,
                    session,
                    body,
                }
            },
        )
}

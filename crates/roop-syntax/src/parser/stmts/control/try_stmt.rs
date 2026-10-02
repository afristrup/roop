use crate::{Block, StmtKind, Token};
use crate::{Err, TokenInput, expr, place};
use chumsky::prelude::*;

/// `try { ... } catch_rollback { ... }`, optionally followed by `-> outcome;`.
pub fn try_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    let outcome = just(Token::Arrow)
        .ignore_then(place(expr()))
        .then_ignore(just(Token::Semi));
    just(Token::Try)
        .ignore_then(blk.clone())
        .then_ignore(just(Token::CatchRollback))
        .then(blk)
        .then(outcome.or_not())
        .map(|((body, handler), outcome)| StmtKind::Try {
            body,
            handler,
            outcome,
        })
}

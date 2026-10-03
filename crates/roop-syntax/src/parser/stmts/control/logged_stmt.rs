use crate::{Block, StmtKind, Token};
use crate::{Err, TokenInput, expr, place};
use chumsky::prelude::*;

/// `logged history { ... }`
pub fn logged_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Logged)
        .ignore_then(place(expr()))
        .then(blk)
        .map(|(history, body)| StmtKind::Logged { history, body })
}

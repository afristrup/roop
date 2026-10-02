use crate::{Block, StmtKind, Token};
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn try_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Try)
        .ignore_then(blk.clone())
        .then_ignore(just(Token::CatchRollback))
        .then(blk)
        .map(|(body, handler)| StmtKind::Try { body, handler })
}

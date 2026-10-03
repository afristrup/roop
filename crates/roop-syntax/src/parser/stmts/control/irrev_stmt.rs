use crate::{Block, StmtKind, Token};
use crate::{Err, TokenInput};
use chumsky::prelude::*;

/// `irrev { ... }`
pub fn irrev_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    just(Token::Irrev).ignore_then(blk).map(StmtKind::Irrev)
}

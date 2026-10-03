use crate::{Block, Stmt, Token, entries, fold_entries};
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn block<'a, I: TokenInput<'a>>(
    stmt: impl Parser<'a, I, Stmt, Err<'a>> + Clone,
) -> impl Parser<'a, I, Block, Err<'a>> + Clone {
    entries(stmt)
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map_with(|entries, e| {
            let span = e.span();
            Block {
                stmts: fold_entries(entries, span.end),
                span,
            }
        })
}

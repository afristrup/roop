use crate::{Block, Stmt, Token};
use crate::{Err, TokenInput};
use chumsky::prelude::*;

pub fn block<'a, I: TokenInput<'a>>(
    stmt: impl Parser<'a, I, Stmt, Err<'a>> + Clone,
) -> impl Parser<'a, I, Block, Err<'a>> + Clone {
    stmt.repeated()
        .collect()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map_with(|stmts, e| Block {
            stmts,
            span: e.span(),
        })
}

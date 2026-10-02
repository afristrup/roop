use crate::{Block, MatchArm, StmtKind, Token};
use crate::{Err, TokenInput, expr, pattern};
use chumsky::prelude::*;

pub fn match_stmt<'a, I: TokenInput<'a>>(
    blk: impl Parser<'a, I, Block, Err<'a>> + Clone,
) -> impl Parser<'a, I, StmtKind, Err<'a>> + Clone {
    let arm = pattern()
        .then_ignore(just(Token::FatArrow))
        .then(blk)
        .then_ignore(just(Token::Assert))
        .then(expr())
        .then_ignore(just(Token::Semi))
        .map(|((pattern, body), exit)| MatchArm {
            pattern,
            body,
            exit,
        });
    just(Token::Match)
        .ignore_then(expr())
        .then(
            arm.repeated()
                .at_least(1)
                .collect()
                .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|(scrutinee, arms)| StmtKind::Match { scrutinee, arms })
}

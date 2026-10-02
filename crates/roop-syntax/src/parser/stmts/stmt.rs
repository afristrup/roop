use crate::{Block, Stmt, StmtKind, Token, UpdateOp};
use crate::{
    Err, TokenInput, attr, block, borrow_stmt, expr, ident, match_stmt, place, try_stmt, ty,
};
use chumsky::prelude::*;

pub fn stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Stmt, Err<'a>> + Clone {
    recursive(|stmt| {
        let blk = block(stmt);
        let match_ = match_stmt(blk.clone());
        let borrow = borrow_stmt(blk.clone());
        let try_ = try_stmt(blk.clone());
        let semi = just(Token::Semi);

        let update_op = select! {
            Token::PlusEq => UpdateOp::Add,
            Token::MinusEq => UpdateOp::Sub,
            Token::CaretEq => UpdateOp::Xor,
        };
        let update = place(expr())
            .then(update_op)
            .then(expr())
            .then_ignore(semi.clone())
            .map(|((target, op), value)| StmtKind::Update { target, op, value });
        let swap = place(expr())
            .then_ignore(just(Token::Swap))
            .then(place(expr()))
            .then_ignore(semi.clone())
            .map(|(a, b)| StmtKind::Swap(a, b));

        let if_ = just(Token::If)
            .ignore_then(expr())
            .then(blk.clone())
            .then(
                just(Token::Else)
                    .ignore_then(blk.clone())
                    .or_not()
                    .map(Option::unwrap_or_default),
            )
            .then_ignore(just(Token::Fi))
            .then(expr())
            .then_ignore(semi.clone())
            .map(|(((cond, then_block), else_block), exit)| StmtKind::If {
                cond,
                then_block,
                else_block,
                exit,
            });
        let from = just(Token::From)
            .ignore_then(expr())
            .then(blk.clone())
            .then(
                just(Token::Loop)
                    .ignore_then(blk.clone())
                    .or_not()
                    .map(Option::unwrap_or_default),
            )
            .then_ignore(just(Token::Until))
            .then(expr())
            .then_ignore(semi.clone())
            .map(
                |(((entry, body), step), until): (((_, Block), Block), _)| StmtKind::From {
                    entry,
                    body,
                    step,
                    until,
                },
            );
        let ancilla = just(Token::Ancilla)
            .ignore_then(ident())
            .then_ignore(just(Token::Colon))
            .then(ty())
            .then_ignore(just(Token::Assign))
            .then(expr())
            .then(blk)
            .map(|(((name, ty), init), body)| StmtKind::Ancilla {
                name,
                ty,
                init,
                body,
            });

        let args = expr()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::LParen), just(Token::RParen));
        let call = just(Token::Call)
            .ignore_then(ident())
            .then(args.clone())
            .then_ignore(semi.clone())
            .map(|(callee, args)| StmtKind::Call { callee, args });
        let uncall = just(Token::Uncall)
            .ignore_then(ident())
            .then(args)
            .then_ignore(semi)
            .map(|(callee, args)| StmtKind::Uncall { callee, args });

        let kind = update
            .or(swap)
            .or(if_)
            .or(match_)
            .or(try_)
            .or(borrow)
            .or(from)
            .or(ancilla)
            .or(call)
            .or(uncall);

        attr()
            .repeated()
            .collect()
            .then(kind)
            .map_with(|(attrs, kind), e| Stmt {
                attrs,
                kind,
                span: e.span(),
            })
    })
}

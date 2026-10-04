use crate::{BinOp, Block, Expr, Place, Stmt, StmtKind, Token, UpdateOp, comma_list};
use crate::{
    Err, TokenInput, attr, block, borrow_stmt, chan_stmt, expr, ident, irrev_stmt, keep_stmt,
    logged_stmt, match_stmt, overwrite_stmt, place, pop_stmt, push_stmt, recv_stmt, send_stmt,
    try_stmt, ty,
};
use chumsky::prelude::*;

pub fn stmt<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Stmt, Err<'a>> + Clone {
    recursive(|stmt| {
        let blk = block(stmt);
        let match_ = match_stmt(blk.clone());
        let borrow = borrow_stmt(blk.clone());
        let try_ = try_stmt(blk.clone());
        let irrev = irrev_stmt(blk.clone());
        let logged = logged_stmt(blk.clone());
        let chan = chan_stmt(blk.clone());
        let plain_block = blk.clone().map(StmtKind::Block);
        let semi = just(Token::Semi);

        let update_op = select! {
            Token::PlusEq => UpdateOp::Add,
            Token::MinusEq => UpdateOp::Sub,
            Token::CaretEq => UpdateOp::Xor,
            Token::StarEq => UpdateOp::Mul,
            Token::SlashEq => UpdateOp::Div,
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
        let term = select! { Token::Int(s) => s }
            .try_map(|s, span| {
                s.parse::<i64>()
                    .map(Expr::Int)
                    .map_err(|e| Rich::custom(span, e.to_string()))
            })
            .or(ident().map(|name| Expr::Place(Place::Var(name))));
        let length = term.clone().foldl(
            just(Token::Star).ignore_then(term).repeated(),
            |left, right| Expr::Binary(Box::new(left), BinOp::Mul, Box::new(right)),
        );
        let generic_args = comma_list(length)
            .delimited_by(just(Token::Lt), just(Token::Gt))
            .or_not()
            .map(Option::unwrap_or_default);
        let call = just(Token::Call)
            .ignore_then(ident())
            .then(generic_args.clone())
            .then(args.clone())
            .then_ignore(semi.clone())
            .map(|((callee, generics), args)| StmtKind::Call {
                callee,
                generics,
                args,
            });
        let uncall = just(Token::Uncall)
            .ignore_then(ident())
            .then(generic_args)
            .then(args)
            .then_ignore(semi.clone())
            .map(|((callee, generics), args)| StmtKind::Uncall {
                callee,
                generics,
                args,
            });

        let expect = select! { Token::Ident("expect") => () }
            .ignore_then(expr())
            .then_ignore(semi.clone())
            .map(|cond| StmtKind::If {
                cond,
                then_block: Block::default(),
                else_block: Block::default(),
                exit: Expr::Bool(true),
            });

        let kind = update
            .or(swap)
            .or(if_)
            .or(match_)
            .or(try_)
            .or(borrow)
            .or(from)
            .or(ancilla)
            .or(call)
            .or(uncall)
            .or(expect)
            .or(chan)
            .or(send_stmt())
            .or(recv_stmt())
            .or(irrev)
            .or(logged)
            .or(push_stmt())
            .or(pop_stmt())
            .or(keep_stmt())
            .or(overwrite_stmt())
            .or(plain_block);

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

use crate::{BinOp, Expr, Token, UnOp};
use crate::{Err, TokenInput, binary_level, place, variant};
use chumsky::prelude::*;

pub fn expr<'a, I: TokenInput<'a>>() -> impl Parser<'a, I, Expr, Err<'a>> + Clone {
    recursive(|expr| {
        let int = select! { Token::Int(s) => s }.try_map(|s, span| {
            s.parse::<i64>()
                .map(Expr::Int)
                .map_err(|e| Rich::custom(span, e.to_string()))
        });
        let float = select! { Token::Float(s) => s }.try_map(|s, span| {
            s.parse::<f64>()
                .map(Expr::Float)
                .map_err(|e| Rich::custom(span, e.to_string()))
        });
        let empty = just(Token::Empty).to(Expr::Empty);
        let boolean =
            select! { Token::True => Expr::Bool(true), Token::False => Expr::Bool(false) };
        let group = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));
        let atom = int
            .or(float)
            .or(boolean)
            .or(empty)
            .or(variant().map(|(e, v)| Expr::Variant(e, v)))
            .or(place(expr).map(Expr::Place))
            .or(group);

        let unary_op = select! { Token::Minus => UnOp::Neg, Token::Bang => UnOp::Not };
        let unary = unary_op
            .repeated()
            .foldr(atom, |op, e| Expr::Unary(op, Box::new(e)));

        let mul = binary_level(
            unary,
            select! { Token::Star => BinOp::Mul, Token::Slash => BinOp::Div, Token::Percent => BinOp::Rem },
        );
        let add = binary_level(
            mul,
            select! { Token::Plus => BinOp::Add, Token::Minus => BinOp::Sub },
        );
        let cmp = binary_level(
            add,
            select! {
                Token::Lt => BinOp::Lt, Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt, Token::Ge => BinOp::Ge,
            },
        );
        let eq = binary_level(
            cmp,
            select! { Token::EqEq => BinOp::Eq, Token::NotEq => BinOp::Ne },
        );
        let and = binary_level(eq, select! { Token::AmpAmp => BinOp::And });
        binary_level(and, select! { Token::PipePipe => BinOp::Or })
    })
}

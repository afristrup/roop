use crate::{Doc, op_text, parenthesized, precedence, print_expr};
use roop_syntax::Expr;

/// A chain of operators of one precedence, which breaks before an operator
/// when it does not fit. A subexpression that binds looser gets parentheses,
/// and so does a right operand of the same precedence.
pub fn print_binary(expr: &Expr) -> Doc {
    let Expr::Binary(_, top, _) = expr else {
        return print_expr(expr);
    };
    let level = precedence(*top);
    let mut rest = Vec::new();
    let mut head = expr;
    while let Expr::Binary(left, op, right) = head {
        if precedence(*op) != level {
            break;
        }
        rest.push((*op, &**right));
        head = left;
    }
    let operand = |e: &Expr, right: bool| match e {
        Expr::Binary(_, op, _)
            if precedence(*op) < level || (right && precedence(*op) == level) =>
        {
            parenthesized(print_expr(e))
        }
        _ => print_expr(e),
    };
    let tail = rest.iter().rev().map(|(op, right)| {
        Doc::concat(vec![
            Doc::Line,
            Doc::text(format!("{} ", op_text(*op))),
            operand(right, true),
        ])
    });
    Doc::group(Doc::concat(vec![
        operand(head, false),
        Doc::nest(Doc::concat(tail.collect())),
    ]))
}

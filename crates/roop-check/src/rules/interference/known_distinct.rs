use crate::Facts;
use roop_syntax::{BinOp, Expr};

/// Whether two index expressions provably differ: different constants, the same
/// expression at different constant offsets, or a condition among `facts`
/// that says so.
pub fn known_distinct(facts: Option<&Facts>, i: &Expr, j: &Expr) -> bool {
    if let (Expr::Int(a), Expr::Int(b)) = (i, j) {
        return a != b;
    }
    if offsets_differ(i, j) {
        return true;
    }
    let mut node = facts;
    while let Some(fact) = node {
        if separates(fact.cond, i, j) {
            return true;
        }
        node = fact.parent;
    }
    false
}

fn separates(cond: &Expr, i: &Expr, j: &Expr) -> bool {
    match cond {
        Expr::Binary(l, BinOp::And, r) => separates(l, i, j) || separates(r, i, j),
        Expr::Binary(l, BinOp::Lt | BinOp::Gt | BinOp::Ne, r) => {
            (**l == *i && **r == *j) || (**l == *j && **r == *i)
        }
        _ => false,
    }
}

/// The part that is not a constant, and the constant offset.
fn split(expr: &Expr) -> (Option<&Expr>, i64) {
    match expr {
        Expr::Int(c) => (None, *c),
        Expr::Binary(base, BinOp::Add, offset) => match **offset {
            Expr::Int(c) => (Some(base), c),
            _ => (Some(expr), 0),
        },
        Expr::Binary(base, BinOp::Sub, offset) => match **offset {
            Expr::Int(c) => (Some(base), c.wrapping_neg()),
            _ => (Some(expr), 0),
        },
        _ => (Some(expr), 0),
    }
}

fn offsets_differ(i: &Expr, j: &Expr) -> bool {
    let ((base_i, off_i), (base_j, off_j)) = (split(i), split(j));
    base_i == base_j && off_i != off_j
}

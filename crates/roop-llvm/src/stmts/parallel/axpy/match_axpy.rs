use crate::{Axpy, element, unit_loop};
use roop_syntax::{BinOp, Block, Expr, Place, StmtKind, UpdateOp};

/// Recognizes a `daxpy` loop over `i`: `y[i] += alpha * x[i]`.
pub fn match_axpy<'a>(
    entry: &'a Expr,
    body: &'a Block,
    step: &'a Block,
    until: &'a Expr,
) -> Option<Axpy<'a>> {
    let (i, len) = unit_loop(entry, step, until)?;
    let [update] = body.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Update {
        target,
        op: UpdateOp::Add,
        value: Expr::Binary(alpha, BinOp::Mul, x_cell),
    } = &update.kind
    else {
        return None;
    };
    let (Expr::Place(Place::Var(alpha)), Expr::Place(x_cell)) = (&**alpha, &**x_cell) else {
        return None;
    };
    let (y, x) = (element(target, i)?, element(x_cell, i)?);
    let distinct = y != x && y != alpha && x != alpha;
    distinct.then_some(Axpy { y, x, alpha, len })
}

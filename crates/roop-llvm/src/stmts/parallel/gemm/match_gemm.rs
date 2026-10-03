use crate::{Gemm, cell, nested_loop, unit_loop};
use roop_syntax::{BinOp, Block, Expr, Place, StmtKind, UpdateOp};

/// Recognizes the body of a `dgemm` row loop over `i`: the loops over `l` and
/// `j` around `c[i][j] += alpha * a[i][l] * b[l][j]`.
pub fn match_gemm<'a>(
    entry: &'a Expr,
    body: &'a Block,
    step: &'a Block,
    until: &'a Expr,
) -> Option<Gemm<'a>> {
    let (i, rows) = unit_loop(entry, step, until)?;
    let (l, inner, over_l) = nested_loop(body)?;
    let (j, cols, over_j) = nested_loop(over_l)?;
    let [update] = over_j.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Update {
        target,
        op: UpdateOp::Add,
        value: Expr::Binary(scaled, BinOp::Mul, b_cell),
    } = &update.kind
    else {
        return None;
    };
    let Expr::Binary(alpha, BinOp::Mul, a_cell) = &**scaled else {
        return None;
    };
    let (Expr::Place(Place::Var(alpha)), Expr::Place(a_cell), Expr::Place(b_cell)) =
        (&**alpha, &**a_cell, &**b_cell)
    else {
        return None;
    };
    let (c, a, b) = (
        cell(target, i, j)?,
        cell(a_cell, i, l)?,
        cell(b_cell, l, j)?,
    );
    let names = [c, a, b, alpha.as_str()];
    let distinct = (0..4).all(|x| (x + 1..4).all(|y| names[x] != names[y]));
    distinct.then_some(Gemm {
        c,
        a,
        b,
        alpha,
        rows,
        inner,
        cols,
    })
}

use crate::{Q12Layout, Q12Matmul, cell, nested_loop, unit_loop};
use roop_syntax::{BinOp, Block, Expr, Place, StmtKind, UpdateOp};

fn operands<'a>(
    a: &'a Place,
    b: &'a Place,
    (i, k, j): (&str, &str, &str),
) -> Option<(&'a str, &'a str, Q12Layout)> {
    let nn = || Some((cell(a, i, j)?, cell(b, j, k)?, Q12Layout::NN));
    let nt = || Some((cell(a, i, j)?, cell(b, k, j)?, Q12Layout::NT));
    let tn = || Some((cell(a, j, i)?, cell(b, j, k)?, Q12Layout::TN));
    nn().or_else(nt).or_else(tn)
}

/// Recognizes the body of a `q12` product's row loop over `i`: the loops over
/// `k` and `j` around `c[i][k] += a[..][..] * b[..][..] / 4096`, which is what
/// the `q12` einsums `qmatmul`, `qmatmul_nt` and `qmatmul_tn` expand to.
pub fn match_q12_matmul<'a>(
    entry: &'a Expr,
    body: &'a Block,
    step: &'a Block,
    until: &'a Expr,
) -> Option<Q12Matmul<'a>> {
    let (i, rows) = unit_loop(entry, step, until)?;
    let (k, cols, over_k) = nested_loop(body)?;
    let (j, inner, over_j) = nested_loop(over_k)?;
    let [update] = over_j.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Update {
        target,
        op: UpdateOp::Add,
        value: Expr::Binary(product, BinOp::Div, scale),
    } = &update.kind
    else {
        return None;
    };
    let (Expr::Int(4096), Expr::Binary(a_cell, BinOp::Mul, b_cell)) = (&**scale, &**product) else {
        return None;
    };
    let (Expr::Place(a_cell), Expr::Place(b_cell)) = (&**a_cell, &**b_cell) else {
        return None;
    };
    let c = cell(target, i, k)?;
    let (a, b, layout) = operands(a_cell, b_cell, (i, k, j))?;
    let names = [c, a, b];
    let distinct = (0..3).all(|x| (x + 1..3).all(|y| names[x] != names[y]));
    distinct.then_some(Q12Matmul {
        c,
        a,
        b,
        rows,
        inner,
        cols,
        layout,
    })
}

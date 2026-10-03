use crate::{IntLayout, IntMatmul, cell, nested_loop, unit_loop};
use roop_syntax::{BinOp, Block, Expr, Place, StmtKind, UpdateOp};

fn operands<'a>(
    a: &'a Place,
    b: &'a Place,
    (i, k, j): (&str, &str, &str),
) -> Option<(&'a str, &'a str, IntLayout)> {
    let nn = || Some((cell(a, i, j)?, cell(b, j, k)?, IntLayout::NN));
    let nt = || Some((cell(a, i, j)?, cell(b, k, j)?, IntLayout::NT));
    let tn = || Some((cell(a, j, i)?, cell(b, j, k)?, IntLayout::TN));
    nn().or_else(nt).or_else(tn)
}

/// The two cells of a product, and whether it is divided by 4096.
fn product(value: &Expr) -> Option<(&Place, &Place, bool)> {
    let (inner, scaled) = match value {
        Expr::Binary(inner, BinOp::Div, scale) if **scale == Expr::Int(4096) => (&**inner, true),
        other => (other, false),
    };
    let Expr::Binary(a, BinOp::Mul, b) = inner else {
        return None;
    };
    let (Expr::Place(a), Expr::Place(b)) = (&**a, &**b) else {
        return None;
    };
    Some((a, b, scaled))
}

/// Recognizes the body of an integer product's row loop over `i`: the loops
/// over `k` and `j` around `c[i][k] += a[..][..] * b[..][..]`, with the product
/// divided by 4096 or not. These are what the `q12` einsums (`qmatmul`,
/// `qmatmul_nt`, `qmatmul_tn`) and the `i64` ones (`imatmul`, ...) expand to.
pub fn match_int_matmul<'a>(
    entry: &'a Expr,
    body: &'a Block,
    step: &'a Block,
    until: &'a Expr,
) -> Option<IntMatmul<'a>> {
    let (i, rows) = unit_loop(entry, step, until)?;
    let (k, cols, over_k) = nested_loop(body)?;
    let (j, inner, over_j) = nested_loop(over_k)?;
    let [update] = over_j.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Update {
        target,
        op: UpdateOp::Add,
        value,
    } = &update.kind
    else {
        return None;
    };
    let (a_cell, b_cell, scaled) = product(value)?;
    let c = cell(target, i, k)?;
    let (a, b, layout) = operands(a_cell, b_cell, (i, k, j))?;
    let names = [c, a, b];
    let distinct = (0..3).all(|x| (x + 1..3).all(|y| names[x] != names[y]));
    distinct.then_some(IntMatmul {
        c,
        a,
        b,
        rows,
        inner,
        cols,
        layout,
        scaled,
    })
}

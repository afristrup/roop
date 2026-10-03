use crate::{BinOp, Block, Expr, Place, StmtKind, UpdateOp};

/// A `from v == lo ... loop { v += step; } until v == hi` loop. The body runs
/// for `lo, lo + step, ..., hi`, both ends included.
pub struct CountedLoop<'a> {
    pub var: &'a str,
    pub lo: &'a Expr,
    pub hi: &'a Expr,
    pub step: i64,
}

pub fn counted_loop<'a>(
    entry: &'a Expr,
    step: &'a Block,
    until: &'a Expr,
) -> Option<CountedLoop<'a>> {
    let (var, lo) = var_equals(entry)?;
    let (end_var, hi) = var_equals(until)?;
    let [only] = step.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Update {
        target: Place::Var(stepped),
        op: UpdateOp::Add,
        value: Expr::Int(by),
    } = &only.kind
    else {
        return None;
    };
    (end_var == var && stepped == var && *by > 0).then_some(CountedLoop {
        var,
        lo,
        hi,
        step: *by,
    })
}

fn var_equals(expr: &Expr) -> Option<(&str, &Expr)> {
    match expr {
        Expr::Binary(lhs, BinOp::Eq, rhs) => match &**lhs {
            Expr::Place(Place::Var(name)) => Some((name, rhs)),
            _ => None,
        },
        _ => None,
    }
}

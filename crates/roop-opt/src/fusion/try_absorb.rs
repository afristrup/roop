use crate::{
    Fused, declares, expr_mentions, mentions, merge_target, par_loop, rename_block, synth,
};
use roop_check::check_parallel_body;
use roop_syntax::{Attr, BinOp, Expr, Place, Span, Stmt, StmtKind, UpdateOp};

/// Merges `next` into the fused loop when it has the same iteration space and
/// the merged body is still safe to run iteration by iteration. Legality is
/// exactly the parallel-loop rule, applied to the combined body.
pub fn try_absorb(fused: &mut Fused, next: &Stmt) -> bool {
    let (Some(a), Some(b)) = (par_loop(&fused.head), par_loop(next)) else {
        return false;
    };
    let same_space = a.lo == b.lo && a.hi == b.hi && a.step == b.step;
    let distinct_var = a.var != b.var;
    let bounds_untouched = !expr_mentions(a.lo, b.var) && !expr_mentions(a.hi, b.var);
    let names_free = !mentions(b.body, a.var)
        && !declares(b.body, a.var)
        && !declares(b.body, b.var)
        && !declares(a.body, a.var);
    let Some(target) = merge_target(a.target, b.target) else {
        return false;
    };
    if !(same_space && distinct_var && bounds_untouched && names_free) {
        return false;
    }

    let mut body = a.body.clone();
    body.stmts.extend(rename_block(b.body, b.var, a.var).stmts);
    let span = Span::from(fused.head.span.start..next.span.end);
    if check_parallel_body(a.var, a.lo, a.hi, &body, span).is_err() {
        return false;
    }

    let (lo, hi, var_b) = (a.lo.clone(), a.hi.clone(), b.var.to_string());
    let StmtKind::From {
        entry, step, until, ..
    } = fused.head.kind.clone()
    else {
        return false;
    };
    let at_lo = Expr::Binary(
        Box::new(Expr::Place(Place::Var(var_b.clone()))),
        BinOp::Eq,
        Box::new(lo.clone()),
    );
    fused.prologue.push(synth(
        StmtKind::If {
            cond: at_lo.clone(),
            then_block: Default::default(),
            else_block: Default::default(),
            exit: at_lo,
        },
        next.span,
    ));
    fused.epilogue.push(synth(
        StmtKind::Update {
            target: Place::Var(var_b),
            op: UpdateOp::Add,
            value: Expr::Binary(Box::new(hi), BinOp::Sub, Box::new(lo)),
        },
        next.span,
    ));
    fused.head = Stmt {
        attrs: vec![Attr::Parallel { target }],
        kind: StmtKind::From {
            entry,
            body,
            step,
            until,
        },
        span,
    };
    true
}

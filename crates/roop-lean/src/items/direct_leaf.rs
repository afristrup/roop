use roop_syntax::{Block, Expr, FnDef, Place, StmtKind, UpdateOp, counted_loop};

pub fn counted_ancilla(def: &FnDef) -> Option<(usize, i64)> {
    if !def.einsum_generated {
        return None;
    }
    let [stmt] = def.body.stmts.as_slice() else {
        return None;
    };
    let StmtKind::Ancilla {
        name,
        init: Expr::Int(0),
        body,
        ..
    } = &stmt.kind
    else {
        return None;
    };
    let [loop_stmt, cleanup] = body.stmts.as_slice() else {
        return None;
    };
    let StmtKind::From {
        entry, step, until, ..
    } = &loop_stmt.kind
    else {
        return None;
    };
    let counted = counted_loop(entry, step, until)?;
    let StmtKind::Update {
        target: Place::Var(target),
        op: UpdateOp::Sub,
        value: Expr::Int(amount),
    } = &cleanup.kind
    else {
        return None;
    };
    (counted.var == name
        && matches!(counted.lo, Expr::Int(0))
        && counted.step == 1
        && matches!(counted.hi, Expr::Int(limit) if limit == amount)
        && target == name)
        .then_some((loop_stmt.span.start, *amount))
}

/// Whether a function can get a no-ancilla contract by structural splitting.
pub fn direct_leaf(def: &FnDef) -> bool {
    direct_block(&def.body)
}

fn direct_block(block: &Block) -> bool {
    block.stmts.iter().all(|stmt| match &stmt.kind {
        StmtKind::Update {
            target: Place::Var(_),
            op: UpdateOp::Add | UpdateOp::Sub | UpdateOp::Xor,
            value,
        } => direct_expr(value),
        StmtKind::Swap(Place::Var(_), Place::Var(_)) => true,
        StmtKind::If {
            cond,
            then_block,
            else_block,
            ..
        } => direct_expr(cond) && direct_block(then_block) && direct_block(else_block),
        _ => false,
    })
}

fn direct_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Unary(_, expr) | Expr::Cast(expr, _) => direct_expr(expr),
        Expr::Binary(lhs, _, rhs) => direct_expr(lhs) && direct_expr(rhs),
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::Byte(_)
        | Expr::Place(Place::Var(_)) => true,
        _ => false,
    }
}

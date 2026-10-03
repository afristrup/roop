use crate::stmt_writes;
use roop_syntax::{BinOp, Expr, Place, Stmt, StmtKind, UpdateOp, counted_loop};
use std::collections::HashSet;

/// A counted loop over `name` as the one update it amounts to: the counter
/// starts at `lo` and ends at `hi`, so the loop is `name += hi - lo`, as long
/// as the body leaves the counter alone.
pub fn counted_net(stmt: &Stmt, name: &str) -> Option<Stmt> {
    let StmtKind::From {
        entry,
        body,
        step,
        until,
    } = &stmt.kind
    else {
        return None;
    };
    let counted = counted_loop(entry, step, until).filter(|c| c.var == name)?;
    let mut writes = HashSet::new();
    body.stmts
        .iter()
        .for_each(|s| stmt_writes(s, &mut writes, None));
    if writes.contains(name) {
        return None;
    }
    let value = match counted.lo {
        Expr::Int(0) => counted.hi.clone(),
        lo => Expr::Binary(
            Box::new(counted.hi.clone()),
            BinOp::Sub,
            Box::new(lo.clone()),
        ),
    };
    Some(Stmt {
        attrs: Vec::new(),
        kind: StmtKind::Update {
            target: Place::Var(name.to_string()),
            op: UpdateOp::Add,
            value,
        },
        span: stmt.span,
    })
}

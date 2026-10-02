use super::{expr_places, push_place, stmt_exprs};
use roop_syntax::{Place, Stmt, StmtKind};

/// Every place the statement itself touches, excluding nested blocks.
pub fn stmt_places(stmt: &Stmt) -> Vec<&Place> {
    let mut out = Vec::new();
    match &stmt.kind {
        StmtKind::Update { target, .. } => push_place(target, &mut out),
        StmtKind::Swap(a, b) => {
            push_place(a, &mut out);
            push_place(b, &mut out);
        }
        StmtKind::Borrow { source, .. } => push_place(source, &mut out),
        _ => {}
    }
    for expr in stmt_exprs(stmt) {
        expr_places(expr, &mut out);
    }
    out
}

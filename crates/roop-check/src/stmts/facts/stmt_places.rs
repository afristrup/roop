use crate::{expr_places, push_place, stmt_exprs};
use roop_syntax::{Place, Stmt, StmtKind};

/// Every place the statement itself touches, excluding nested blocks.
pub fn stmt_places(stmt: &Stmt) -> Vec<&Place> {
    let mut out = Vec::new();
    match &stmt.kind {
        StmtKind::Update { target, .. } | StmtKind::Overwrite { target, .. } => {
            push_place(target, &mut out)
        }
        StmtKind::Swap(a, b) => {
            push_place(a, &mut out);
            push_place(b, &mut out);
        }
        StmtKind::Borrow { source, .. } => push_place(source, &mut out),
        StmtKind::Send { source, .. } => push_place(source, &mut out),
        StmtKind::Recv { target, .. } => push_place(target, &mut out),
        StmtKind::Push { stack, source } => {
            push_place(stack, &mut out);
            push_place(source, &mut out);
        }
        StmtKind::Pop { stack, target } => {
            push_place(stack, &mut out);
            push_place(target, &mut out);
        }
        StmtKind::Keep(place) => push_place(place, &mut out),
        StmtKind::Logged { history, .. } => push_place(history, &mut out),
        StmtKind::Try {
            outcome: Some(outcome),
            ..
        } => push_place(outcome, &mut out),
        _ => {}
    }
    for expr in stmt_exprs(stmt) {
        expr_places(expr, &mut out);
    }
    out
}

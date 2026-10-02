use super::{expr_places, push_place};
use roop_syntax::{Expr, Place, Stmt, StmtKind};

/// Every place the statement itself touches, excluding nested blocks.
pub fn stmt_places(stmt: &Stmt) -> Vec<&Place> {
    let mut out = Vec::new();
    let exprs: Vec<&Expr> = match &stmt.kind {
        StmtKind::Update { target, value, .. } => {
            push_place(target, &mut out);
            vec![value]
        }
        StmtKind::Swap(a, b) => {
            push_place(a, &mut out);
            push_place(b, &mut out);
            Vec::new()
        }
        StmtKind::If { cond, exit, .. } => vec![cond, exit],
        StmtKind::From { entry, until, .. } => vec![entry, until],
        StmtKind::Match { scrutinee, arms } => std::iter::once(scrutinee)
            .chain(arms.iter().map(|arm| &arm.exit))
            .collect(),
        StmtKind::Ancilla { init, .. } => vec![init],
        StmtKind::Call { args, .. } | StmtKind::Uncall { args, .. } => args.iter().collect(),
        StmtKind::Borrow { source, .. } => {
            push_place(source, &mut out);
            Vec::new()
        }
        StmtKind::Try { .. } => Vec::new(),
    };
    for expr in exprs {
        expr_places(expr, &mut out);
    }
    out
}

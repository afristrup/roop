use roop_syntax::{Expr, Stmt, StmtKind};

/// Every expression the statement itself evaluates, excluding nested blocks.
pub fn stmt_exprs(stmt: &Stmt) -> Vec<&Expr> {
    match &stmt.kind {
        StmtKind::Update { value, .. } => vec![value],
        StmtKind::If { cond, exit, .. } => vec![cond, exit],
        StmtKind::From { entry, until, .. } => vec![entry, until],
        StmtKind::Match { scrutinee, arms } => std::iter::once(scrutinee)
            .chain(arms.iter().map(|arm| &arm.exit))
            .collect(),
        StmtKind::Ancilla { init, .. } => vec![init],
        StmtKind::Call { args, .. } | StmtKind::Uncall { args, .. } => args.iter().collect(),
        StmtKind::Swap(..) | StmtKind::Borrow { .. } | StmtKind::Try { .. } => Vec::new(),
    }
}

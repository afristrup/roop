use roop_syntax::{Stmt, StmtKind};

/// Whether a statement is one of the invertible ones that run once, with no
/// control flow in it.
pub fn is_straight_line(stmt: &Stmt) -> bool {
    matches!(
        stmt.kind,
        StmtKind::Update { .. }
            | StmtKind::Swap(..)
            | StmtKind::Call { .. }
            | StmtKind::Uncall { .. }
            | StmtKind::Send { .. }
            | StmtKind::Recv { .. }
            | StmtKind::Push { .. }
            | StmtKind::Pop { .. }
            | StmtKind::Keep(_)
    )
}

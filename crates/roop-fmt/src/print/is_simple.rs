use roop_syntax::{Stmt, StmtKind};

/// A statement that is one line of its own, so a block holding only it may
/// stay on the line of its header.
pub fn is_simple(stmt: &Stmt) -> bool {
    stmt.attrs.is_empty()
        && matches!(
            stmt.kind,
            StmtKind::Update { .. }
                | StmtKind::Swap(..)
                | StmtKind::Overwrite { .. }
                | StmtKind::Push { .. }
                | StmtKind::Pop { .. }
                | StmtKind::Keep(_)
                | StmtKind::Send { .. }
                | StmtKind::Recv { .. }
                | StmtKind::Call { .. }
                | StmtKind::Uncall { .. }
        )
}

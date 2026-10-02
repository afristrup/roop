use roop_syntax::{Block, StmtKind};

/// Whether running the block may fail an exit assertion that an enclosing
/// `try` would catch. Calls count, since the callee may fail. A nested `try`
/// absorbs failures of its own body but not of its handler.
pub fn can_roll_back(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::If { .. } | StmtKind::From { .. } | StmtKind::Match { .. } => true,
        StmtKind::Call { .. } | StmtKind::Uncall { .. } => true,
        StmtKind::Try { handler, .. } => can_roll_back(handler),
        StmtKind::Ancilla { body, .. } => can_roll_back(body),
        StmtKind::Update { .. } | StmtKind::Swap(..) => false,
    })
}

use crate::child_blocks;
use roop_syntax::{Block, StmtKind};

/// Whether irreversible code occurs anywhere in the block: an `irrev` block,
/// an overwrite, or a `try` (rolling back is not itself reversible).
pub fn contains_irrev(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| {
        matches!(
            stmt.kind,
            StmtKind::Irrev(_) | StmtKind::Overwrite { .. } | StmtKind::Try { .. }
        ) || child_blocks(stmt).into_iter().any(contains_irrev)
    })
}

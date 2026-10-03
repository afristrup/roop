use crate::child_blocks;
use roop_syntax::{Block, StmtKind};

/// Whether irreversible code occurs anywhere in the block: an `irrev` block, an
/// overwrite that nothing logs, or a `try` that keeps no outcome (the lost
/// outcome is the information it destroys).
pub fn contains_irrev(block: &Block) -> bool {
    irrev_in(block, false)
}

fn irrev_in(block: &Block, logged: bool) -> bool {
    block.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Irrev(_) | StmtKind::Try { outcome: None, .. } => true,
        StmtKind::Overwrite { .. } if !logged => true,
        StmtKind::Logged { body, .. } => irrev_in(body, true),
        _ => child_blocks(stmt)
            .into_iter()
            .any(|child| irrev_in(child, logged)),
    })
}

use crate::child_blocks;
use roop_syntax::{Block, StmtKind};

/// Whether a `try` occurs anywhere in the block. Rolling back is not itself
/// reversible, so a function containing one has no inverse.
pub fn contains_try(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| {
        matches!(stmt.kind, StmtKind::Try { .. })
            || child_blocks(stmt).into_iter().any(contains_try)
    })
}

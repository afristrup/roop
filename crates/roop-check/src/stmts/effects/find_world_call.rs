use crate::child_blocks;
use roop_syntax::{Block, Span, StmtKind};
use std::collections::HashSet;

/// The first call in the block to a function that changes the world.
pub fn find_world_call(block: &Block, world: &HashSet<&str>) -> Option<(String, Span)> {
    for stmt in &block.stmts {
        if let StmtKind::Call { callee, .. } | StmtKind::Uncall { callee, .. } = &stmt.kind
            && world.contains(callee.as_str())
        {
            return Some((callee.clone(), stmt.span));
        }
        if matches!(stmt.kind, StmtKind::Keep(_)) {
            return Some(("keep".into(), stmt.span));
        }
        for child in child_blocks(stmt) {
            if let Some(hit) = find_world_call(child, world) {
                return Some(hit);
            }
        }
    }
    None
}

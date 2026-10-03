use crate::child_blocks;
use roop_syntax::{Block, StmtKind};
use std::collections::BTreeSet;

/// Every channel name the block sends or receives on, however deeply nested.
pub fn channels_used(block: &Block) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for stmt in &block.stmts {
        if let StmtKind::Send { chan, .. } | StmtKind::Recv { chan, .. } = &stmt.kind {
            out.insert(chan.clone());
        }
        for child in child_blocks(stmt) {
            out.extend(channels_used(child));
        }
    }
    out
}

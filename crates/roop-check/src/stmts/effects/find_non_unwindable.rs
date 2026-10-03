use crate::child_blocks;
use roop_syntax::{Attr, Block, Span, StmtKind};
use std::collections::HashSet;

/// The first thing in the block that a failed `try` could not undo by running
/// the finished statements backward: irreversible code, tasks, parallel loops,
/// channels, a `try` that forgets its outcome, or a call to a function that
/// has any of those.
pub fn find_non_unwindable(
    block: &Block,
    logged: bool,
    blocked: &HashSet<&str>,
) -> Option<(&'static str, Span)> {
    for stmt in &block.stmts {
        let marked = stmt.attrs.iter().find_map(|attr| match attr {
            Attr::Parallel { .. } => Some("a parallel loop"),
            Attr::Concurrent => Some("a concurrent task"),
            Attr::Auto { .. } | Attr::Label(_) => None,
        });
        let found = if marked.is_some() {
            marked
        } else {
            match &stmt.kind {
                StmtKind::Irrev(_) => Some("an irrev block"),
                StmtKind::Overwrite { .. } if !logged => Some("an overwrite outside logged"),
                StmtKind::Try { outcome: None, .. } => Some("a try without an outcome"),
                StmtKind::Chan { .. } | StmtKind::Send { .. } | StmtKind::Recv { .. } => {
                    Some("a channel")
                }
                StmtKind::Call { callee, .. } | StmtKind::Uncall { callee, .. }
                    if blocked.contains(callee.as_str()) =>
                {
                    Some("a call to a function that cannot be undone")
                }
                _ => None,
            }
        };
        if let Some(what) = found {
            return Some((what, stmt.span));
        }
        let inside = logged || matches!(stmt.kind, StmtKind::Logged { .. });
        for child in child_blocks(stmt) {
            if let Some(hit) = find_non_unwindable(child, inside, blocked) {
                return Some(hit);
            }
        }
    }
    None
}

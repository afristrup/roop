use crate::child_blocks;
use roop_syntax::{Stmt, StmtKind};
use std::collections::HashSet;

/// Whether running the statement adds to or takes from the world's history of
/// kept values: it keeps, or it runs a function that does, however deep.
pub fn has_keep_effect(stmt: &Stmt, keeping: &HashSet<&str>) -> bool {
    match &stmt.kind {
        StmtKind::Keep(_) => true,
        StmtKind::Call { callee, .. } | StmtKind::Uncall { callee, .. } => {
            keeping.contains(callee.as_str())
        }
        _ => child_blocks(stmt)
            .into_iter()
            .any(|block| block.stmts.iter().any(|s| has_keep_effect(s, keeping))),
    }
}

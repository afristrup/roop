use crate::child_blocks;
use roop_syntax::{Block, StmtKind};

/// Every call in the block as `(callee, is_uncall)`. With `only_under_try`,
/// only those inside a `try` that keeps an outcome.
pub fn calls_in(block: &Block, only_under_try: bool) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    collect(block, only_under_try, false, &mut out);
    out
}

fn collect(block: &Block, only_under_try: bool, under_try: bool, out: &mut Vec<(String, bool)>) {
    for stmt in &block.stmts {
        if !only_under_try || under_try {
            match &stmt.kind {
                StmtKind::Call { callee, .. } => out.push((callee.clone(), false)),
                StmtKind::Uncall { callee, .. } => out.push((callee.clone(), true)),
                _ => {}
            }
        }
        let inside = under_try
            || matches!(
                stmt.kind,
                StmtKind::Try {
                    outcome: Some(_),
                    ..
                }
            );
        for child in child_blocks(stmt) {
            collect(child, only_under_try, inside, out);
        }
    }
}

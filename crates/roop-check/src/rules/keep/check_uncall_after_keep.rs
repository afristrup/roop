use crate::{CheckError, flat_stmts, has_keep_effect};
use roop_syntax::{Block, StmtKind};
use std::collections::HashSet;

/// The world keeps values on a stack, so `uncall f` takes back what the `call
/// f` before it kept only if nothing was kept in between. Where the earlier
/// call is in the same block and something between them keeps, it is an error.
pub fn check_uncall_after_keep(block: &Block, keeping: &HashSet<&str>) -> Result<(), CheckError> {
    let stmts = flat_stmts(block);
    for (index, stmt) in stmts.iter().enumerate() {
        let StmtKind::Uncall { callee, args, .. } = &stmt.kind else {
            continue;
        };
        if !keeping.contains(callee.as_str()) {
            continue;
        }
        let paired = stmts[..index].iter().rposition(|s| {
            matches!(&s.kind, StmtKind::Call { callee: c, args: a, .. } if c == callee && a == args)
        });
        if let Some(start) = paired
            && stmts[start + 1..index]
                .iter()
                .any(|s| has_keep_effect(s, keeping))
        {
            return Err(CheckError::UncallAfterKeep {
                callee: callee.clone(),
                span: stmt.span,
            });
        }
    }
    Ok(())
}

use crate::{AutoError, blocks_mut, keep_stmt, region_block};
use roop_syntax::{Attr, Block, StmtKind};

/// Ends the labeled loop body or block that follows with a `keep` of `name`.
/// Says whether the label was found.
pub fn keep_in_region(name: &str, region: &str, block: &mut Block) -> Result<bool, AutoError> {
    for stmt in &mut block.stmts {
        if matches!(&stmt.kind, StmtKind::Ancilla { name: inner, .. } if inner == name) {
            continue;
        }
        if stmt
            .attrs
            .iter()
            .any(|a| matches!(a, Attr::Label(l) if l == region))
        {
            let target = region_block(stmt, region)?;
            let at = target.span.end;
            target.stmts.push(keep_stmt(name, at));
            return Ok(true);
        }
        let mut found = false;
        blocks_mut(stmt, &mut |inner| {
            if !found {
                found = keep_in_region(name, region, inner)?;
            }
            Ok(())
        })?;
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

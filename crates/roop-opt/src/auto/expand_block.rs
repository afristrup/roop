use crate::{AutoError, blocks_mut, release, starts_zero};
use roop_syntax::{Attr, Block, StmtKind};

/// Adds the keeps of every `auto ancilla` in the block, inner ones first.
pub fn expand_block(block: &mut Block) -> Result<(), AutoError> {
    for stmt in &mut block.stmts {
        blocks_mut(stmt, &mut |inner| expand_block(inner))?;
        let Some(region) = stmt.attrs.iter().find_map(|a| match a {
            Attr::Auto { region } => Some(region.clone()),
            _ => None,
        }) else {
            continue;
        };
        stmt.attrs.retain(|a| !matches!(a, Attr::Auto { .. }));
        let StmtKind::Ancilla {
            name, init, body, ..
        } = &mut stmt.kind
        else {
            continue;
        };
        if !starts_zero(init) {
            return Err(AutoError::StartsNonZero { name: name.clone() });
        }
        release(name, region.as_deref(), body)?;
    }
    Ok(())
}

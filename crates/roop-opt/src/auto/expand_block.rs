use crate::{AutoError, blocks_mut, release, starts_zero};
use roop_syntax::{Attr, Block, StmtKind};

/// Adds the keeps of every `auto ancilla` in the block, inner ones first.
pub fn expand_block(block: &mut Block) -> Result<(), AutoError> {
    for stmt in &mut block.stmts {
        blocks_mut(stmt, &mut |inner| expand_block(inner))?;
        if !stmt.attrs.contains(&Attr::Auto) {
            continue;
        }
        stmt.attrs.retain(|a| *a != Attr::Auto);
        let StmtKind::Ancilla {
            name, init, body, ..
        } = &mut stmt.kind
        else {
            continue;
        };
        if !starts_zero(init) {
            return Err(AutoError::StartsNonZero { name: name.clone() });
        }
        release(name, body);
    }
    Ok(())
}

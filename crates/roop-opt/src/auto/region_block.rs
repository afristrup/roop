use crate::AutoError;
use roop_syntax::{Block, Stmt, StmtKind};

/// The block that runs once per round of a labeled loop, or the labeled block.
pub fn region_block<'a>(stmt: &'a mut Stmt, region: &str) -> Result<&'a mut Block, AutoError> {
    match &mut stmt.kind {
        StmtKind::From { body, .. } => Ok(body),
        StmtKind::Block(block) => Ok(block),
        _ => Err(AutoError::NotARegion {
            region: region.to_string(),
        }),
    }
}

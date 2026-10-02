use super::{CheckError, Enums, check_exhaustive, check_variants, walk_stmts};
use roop_syntax::{Block, StmtKind};

pub fn check_resolution(enums: &Enums, block: &Block) -> Result<(), CheckError> {
    walk_stmts(block, &mut |stmt| {
        check_variants(enums, stmt)?;
        match &stmt.kind {
            StmtKind::Match { arms, .. } => check_exhaustive(enums, arms, stmt.span),
            _ => Ok(()),
        }
    })
}

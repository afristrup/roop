use crate::{CheckError, child_blocks, place_root, places_overlap, stmt_places};
use roop_syntax::{Block, Place};

/// While a place is borrowed it may only be used through its alias.
pub fn check_borrowed_access(source: &Place, body: &Block) -> Result<(), CheckError> {
    for stmt in &body.stmts {
        if stmt_places(stmt).iter().any(|p| places_overlap(source, p)) {
            return Err(CheckError::BorrowedPlaceUsed {
                var: place_root(source).into(),
                span: stmt.span,
            });
        }
        for child in child_blocks(stmt) {
            check_borrowed_access(source, child)?;
        }
    }
    Ok(())
}

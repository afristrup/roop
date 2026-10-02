use crate::{
    CheckError, check_block, check_borrowed_access, place_index_reads, place_root, stmt_writes,
};
use roop_syntax::{Block, Place, Span};
use std::collections::HashSet;

/// The source place is exclusive to the alias for the whole body, and the
/// variables selecting it must stay put so the borrow can be released.
pub fn check_borrow(source: &Place, body: &Block, span: Span) -> Result<(), CheckError> {
    check_borrowed_access(source, body)?;
    let mut writes = HashSet::new();
    for stmt in &body.stmts {
        stmt_writes(stmt, &mut writes);
    }
    let mut selectors = Vec::new();
    place_index_reads(source, &mut selectors);
    if let Some(moved) = selectors
        .into_iter()
        .map(place_root)
        .find(|root| writes.contains(root))
    {
        return Err(CheckError::BorrowIndexModified {
            var: moved.into(),
            span,
        });
    }
    check_block(body)
}

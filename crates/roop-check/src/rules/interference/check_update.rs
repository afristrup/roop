use crate::{CheckError, expr_places, place_index_reads, place_root, places_overlap};
use roop_syntax::{Expr, Place, Span};

/// Non-interference: the updated place must not overlap anything the update
/// reads, otherwise the update is not injective.
pub fn check_update(target: &Place, value: &Expr, span: Span) -> Result<(), CheckError> {
    let mut reads = Vec::new();
    expr_places(value, &mut reads);
    place_index_reads(target, &mut reads);
    if reads.iter().any(|read| places_overlap(target, read)) {
        return Err(CheckError::SelfReferentialUpdate {
            var: place_root(target).into(),
            span,
        });
    }
    Ok(())
}

use crate::{CheckError, Facts, expr_places, place_index_reads, place_root, places_overlap_given};
use roop_syntax::{Expr, Place, Span};

/// Non-interference: the updated place must not overlap anything the update
/// reads, otherwise the update is not injective. Elements of one array whose
/// indices provably differ, given `facts`, do not overlap.
pub fn check_update(
    target: &Place,
    value: &Expr,
    span: Span,
    facts: Option<&Facts>,
) -> Result<(), CheckError> {
    let mut reads = Vec::new();
    expr_places(value, &mut reads);
    place_index_reads(target, &mut reads);
    if reads
        .iter()
        .any(|read| places_overlap_given(target, read, facts))
    {
        return Err(CheckError::SelfReferentialUpdate {
            var: place_root(target).into(),
            span,
        });
    }
    Ok(())
}

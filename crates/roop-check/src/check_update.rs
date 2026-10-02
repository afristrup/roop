use super::{CheckError, expr_vars, place_index_vars, place_root};
use roop_syntax::{Expr, Place, Span};
use std::collections::HashSet;

/// Non-interference: the updated variable must not occur in anything the
/// update reads, otherwise the update is not injective.
pub fn check_update(target: &Place, value: &Expr, span: Span) -> Result<(), CheckError> {
    let root = place_root(target);
    let mut read = HashSet::new();
    expr_vars(value, &mut read);
    place_index_vars(target, &mut read);
    if read.contains(root) {
        return Err(CheckError::SelfReferentialUpdate {
            var: root.into(),
            span,
        });
    }
    Ok(())
}

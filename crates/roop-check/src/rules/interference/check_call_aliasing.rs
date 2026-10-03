use crate::{
    CheckError, Facts, Mutability, expr_places, place_index_reads, place_root, places_overlap_given,
};
use roop_syntax::{Expr, Span};

/// The arguments of a call must not overlap where one of them is written:
/// `call g(a, a)` with `g(x: &mut, y: &)` would change `y` under the update
/// that reads it, and `uncall` could not put it back. An argument given to a
/// `&mut` parameter may not overlap any other argument, or anything those read.
/// Elements of one array whose indices provably differ do not overlap.
pub fn check_call_aliasing(
    callee: &str,
    args: &[Expr],
    span: Span,
    facts: Option<&Facts>,
    mutability: &Mutability,
) -> Result<(), CheckError> {
    let Some(written) = mutability.get(callee) else {
        return Ok(());
    };
    for (k, arg) in args.iter().enumerate() {
        let (true, Expr::Place(target)) = (written.get(k).copied().unwrap_or(false), arg) else {
            continue;
        };
        let mut others = Vec::new();
        place_index_reads(target, &mut others);
        for other in args
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != k)
            .map(|(_, a)| a)
        {
            expr_places(other, &mut others);
        }
        if others
            .iter()
            .any(|read| places_overlap_given(target, read, facts))
        {
            return Err(CheckError::CallAliasing {
                callee: callee.into(),
                var: place_root(target).into(),
                span,
            });
        }
    }
    Ok(())
}

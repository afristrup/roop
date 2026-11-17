use crate::{Axis, Gate, axes};

/// Two gates commute when on every qubit they share they lie in the algebra
/// of the same Pauli. A sufficient test, not a complete one.
pub fn commutes(a: &Gate, b: &Gate) -> bool {
    let theirs = axes(b);
    axes(a).iter().all(|&(q, axis)| {
        theirs
            .iter()
            .all(|&(r, other)| q != r || (axis == other && axis != Axis::Other))
    })
}

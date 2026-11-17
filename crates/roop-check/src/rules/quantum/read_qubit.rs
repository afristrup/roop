use crate::QubitScope;
use roop_syntax::{Expr, Place};

/// The first qubit that `expr` reads as a value, if any.
pub fn read_qubit(expr: &Expr, scope: &QubitScope) -> Option<String> {
    match expr {
        Expr::Place(place) => read_place_qubit(place, scope),
        Expr::Unary(_, e) | Expr::Cast(e, _) => read_qubit(e, scope),
        Expr::Binary(a, _, b) => read_qubit(a, scope).or_else(|| read_qubit(b, scope)),
        _ => None,
    }
}

/// The first qubit a place reads, as itself or in an index.
pub fn read_place_qubit(place: &Place, scope: &QubitScope) -> Option<String> {
    match place {
        Place::Var(name) => scope.is_qubit(name).then(|| name.clone()),
        Place::Field(base, _) => read_place_qubit(base, scope),
        Place::Index(base, index) => {
            read_place_qubit(base, scope).or_else(|| read_qubit(index, scope))
        }
    }
}

/// The first qubit read in the indices of a place, which a qubit argument may
/// not have: `qs[q]` would read `q`.
pub fn index_qubit(place: &Place, scope: &QubitScope) -> Option<String> {
    match place {
        Place::Var(_) => None,
        Place::Field(base, _) => index_qubit(base, scope),
        Place::Index(base, index) => index_qubit(base, scope).or_else(|| read_qubit(index, scope)),
    }
}

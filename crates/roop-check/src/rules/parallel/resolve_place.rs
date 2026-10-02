use crate::{Binding, place_root, rebase_place};
use roop_syntax::Place;

/// Resolves a place through the enclosing bindings. The bool is true when the
/// place belongs to an iteration-private local.
pub fn resolve_place(place: &Place, scope: &[Binding]) -> (Place, bool) {
    let root = place_root(place);
    match scope.iter().rev().find(|b| match b {
        Binding::Local(name) | Binding::Alias(name, _) => name == root,
    }) {
        Some(Binding::Local(_)) => (place.clone(), true),
        Some(Binding::Alias(name, source)) => (rebase_place(place, name, source), false),
        None => (place.clone(), false),
    }
}

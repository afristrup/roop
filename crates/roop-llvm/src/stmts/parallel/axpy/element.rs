use roop_syntax::{Expr, Place};

/// The vector `v` when `place` is `v[index]`.
pub fn element<'a>(place: &'a Place, index: &str) -> Option<&'a str> {
    let Place::Index(vector, at) = place else {
        return None;
    };
    let Place::Var(name) = &**vector else {
        return None;
    };
    matches!(&**at, Expr::Place(Place::Var(v)) if v == index).then_some(name.as_str())
}

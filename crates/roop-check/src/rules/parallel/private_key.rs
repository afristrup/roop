use crate::is_injective;
use roop_syntax::{Expr, Place};

/// The index expression that makes the place private to one iteration, if
/// any step of its path is indexed injectively by the loop variable.
pub fn private_key<'a>(place: &'a Place, var: &str) -> Option<&'a Expr> {
    match place {
        Place::Var(_) => None,
        Place::Field(base, _) => private_key(base, var),
        Place::Index(base, index) => {
            if is_injective(index, var) {
                Some(index)
            } else {
                private_key(base, var)
            }
        }
    }
}

use crate::{Facts, Step, known_distinct, place_path};
use roop_syntax::{Expr, Place};

/// Like `places_overlap`, but two elements of the same array do not overlap
/// when their indices provably differ.
pub fn places_overlap_given(a: &Place, b: &Place, facts: Option<&Facts>) -> bool {
    let ((root_a, steps_a), (root_b, steps_b)) = (place_path(a), place_path(b));
    if root_a != root_b {
        return false;
    }
    let (mut at_a, mut at_b) = (indices(a).into_iter(), indices(b).into_iter());
    for (x, y) in steps_a.iter().zip(&steps_b) {
        match (x, y) {
            (Step::Field(f), Step::Field(g)) if f != g => return false,
            (Step::Index, Step::Index) => {
                if let (Some(i), Some(j)) = (at_a.next(), at_b.next())
                    && known_distinct(facts, i, j)
                {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// The index expressions along a place, outermost array first.
fn indices(place: &Place) -> Vec<&Expr> {
    match place {
        Place::Var(_) => Vec::new(),
        Place::Field(base, _) => indices(base),
        Place::Index(base, index) => {
            let mut all = indices(base);
            all.push(index);
            all
        }
    }
}

use crate::place_path;
use roop_syntax::Place;

/// Conservative aliasing: distinct fields are disjoint, but any two indices
/// may refer to the same element.
pub fn places_overlap(a: &Place, b: &Place) -> bool {
    let (root_a, steps_a) = place_path(a);
    let (root_b, steps_b) = place_path(b);
    root_a == root_b && steps_a.iter().zip(&steps_b).all(|(x, y)| x == y)
}

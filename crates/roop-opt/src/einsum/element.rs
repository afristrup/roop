use crate::counter;
use roop_syntax::{Expr, Place};

/// `name[i_a][i_b]` for the labels.
pub fn element(name: &str, labels: &[char]) -> Place {
    labels
        .iter()
        .fold(Place::Var(name.to_string()), |place, l| {
            Place::Index(
                Box::new(place),
                Box::new(Expr::Place(Place::Var(counter(*l)))),
            )
        })
}

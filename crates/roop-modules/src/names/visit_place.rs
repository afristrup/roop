use crate::{OnName, visit_expr};
use roop_syntax::Place;

pub fn visit_place(place: &mut Place, on: OnName) {
    match place {
        Place::Var(_) => {}
        Place::Field(inner, _) => visit_place(inner, on),
        Place::Index(inner, index) => {
            visit_place(inner, on);
            visit_expr(index, on);
        }
    }
}

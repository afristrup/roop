use crate::{Place, Visitor, walk_expr};

pub fn walk_place(v: &mut dyn Visitor, place: &mut Place) {
    match place {
        Place::Var(_) => {}
        Place::Field(inner, _) => walk_place(v, inner),
        Place::Index(inner, index) => {
            walk_place(v, inner);
            walk_expr(v, index);
        }
    }
}

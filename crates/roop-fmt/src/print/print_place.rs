use crate::{Doc, print_expr};
use roop_syntax::Place;

pub fn print_place(place: &Place) -> Doc {
    match place {
        Place::Var(name) => Doc::text(name),
        Place::Field(base, field) => {
            Doc::concat(vec![print_place(base), Doc::text(format!(".{field}"))])
        }
        Place::Index(base, index) => Doc::concat(vec![
            print_place(base),
            Doc::text("["),
            print_expr(index),
            Doc::text("]"),
        ]),
    }
}

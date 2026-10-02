use crate::rename_expr;
use roop_syntax::Place;

pub fn rename_place(place: &Place, from: &str, to: &str) -> Place {
    match place {
        Place::Var(name) if name == from => Place::Var(to.into()),
        Place::Var(_) => place.clone(),
        Place::Field(base, f) => Place::Field(Box::new(rename_place(base, from, to)), f.clone()),
        Place::Index(base, i) => Place::Index(
            Box::new(rename_place(base, from, to)),
            Box::new(rename_expr(i, from, to)),
        ),
    }
}

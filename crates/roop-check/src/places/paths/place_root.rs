use roop_syntax::Place;

pub fn place_root(place: &Place) -> &str {
    match place {
        Place::Var(name) => name,
        Place::Field(base, _) | Place::Index(base, _) => place_root(base),
    }
}

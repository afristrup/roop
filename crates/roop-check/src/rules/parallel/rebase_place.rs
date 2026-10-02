use roop_syntax::Place;

/// Replaces the root variable `alias` of `place` by `source`.
pub fn rebase_place(place: &Place, alias: &str, source: &Place) -> Place {
    match place {
        Place::Var(name) if name == alias => source.clone(),
        Place::Var(_) => place.clone(),
        Place::Field(base, f) => {
            Place::Field(Box::new(rebase_place(base, alias, source)), f.clone())
        }
        Place::Index(base, i) => {
            Place::Index(Box::new(rebase_place(base, alias, source)), i.clone())
        }
    }
}

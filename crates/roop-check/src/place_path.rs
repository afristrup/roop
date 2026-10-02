use roop_syntax::Place;

#[derive(PartialEq)]
pub enum Step<'a> {
    Field(&'a str),
    Index,
}

/// Root variable followed by the field and index steps leading to the place.
pub fn place_path(place: &Place) -> (&str, Vec<Step<'_>>) {
    match place {
        Place::Var(name) => (name, Vec::new()),
        Place::Field(base, field) => {
            let (root, mut steps) = place_path(base);
            steps.push(Step::Field(field));
            (root, steps)
        }
        Place::Index(base, _) => {
            let (root, mut steps) = place_path(base);
            steps.push(Step::Index);
            (root, steps)
        }
    }
}

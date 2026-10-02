use super::expr_vars;
use roop_syntax::Place;
use std::collections::HashSet;

pub fn place_vars<'a>(place: &'a Place, out: &mut HashSet<&'a str>) {
    match place {
        Place::Var(name) => {
            out.insert(name);
        }
        Place::Field(base, _) => place_vars(base, out),
        Place::Index(base, index) => {
            place_vars(base, out);
            expr_vars(index, out);
        }
    }
}

use super::expr_places;
use roop_syntax::Place;

pub fn place_index_reads<'a>(place: &'a Place, out: &mut Vec<&'a Place>) {
    match place {
        Place::Var(_) => {}
        Place::Field(base, _) => place_index_reads(base, out),
        Place::Index(base, index) => {
            place_index_reads(base, out);
            expr_places(index, out);
        }
    }
}

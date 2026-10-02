use crate::place_index_reads;
use roop_syntax::Place;

pub fn push_place<'a>(place: &'a Place, out: &mut Vec<&'a Place>) {
    out.push(place);
    place_index_reads(place, out);
}

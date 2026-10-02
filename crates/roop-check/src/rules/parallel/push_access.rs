use crate::{Access, Binding, resolve_place};
use roop_syntax::Place;

pub fn push_access(place: &Place, write: bool, scope: &[Binding], out: &mut Vec<Access>) {
    let (place, local) = resolve_place(place, scope);
    out.push(Access {
        place,
        write,
        local,
    });
}

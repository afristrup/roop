use crate::{CodegenError, Dir, FnGen, gen_place, move_off, move_onto, stack_parts};
use roop_syntax::Place;

/// `push s <- x` moves `x` onto the stack and leaves zero; backward it is a
/// `pop` of the same value into the same place.
pub fn gen_push(
    g: &mut FnGen,
    stack: &Place,
    source: &Place,
    dir: Dir,
) -> Result<(), CodegenError> {
    let stack_slot = gen_place(g, stack)?;
    let parts = stack_parts(g, &stack_slot)?;
    let place = gen_place(g, source)?;
    match dir {
        Dir::Forward => move_onto(g, &parts, &place),
        Dir::Backward => move_off(g, &parts, &place),
    }
}

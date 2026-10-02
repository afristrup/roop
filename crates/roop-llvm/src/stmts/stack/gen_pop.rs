use crate::{CodegenError, Dir, FnGen, gen_place, move_off, move_onto, stack_parts};
use roop_syntax::Place;

/// `pop s -> x` moves the top into `x`, which must be zero; backward it is a
/// `push` of `x` back onto the stack.
pub fn gen_pop(
    g: &mut FnGen,
    stack: &Place,
    target: &Place,
    dir: Dir,
) -> Result<(), CodegenError> {
    let stack_slot = gen_place(g, stack)?;
    let parts = stack_parts(g, &stack_slot)?;
    let place = gen_place(g, target)?;
    match dir {
        Dir::Forward => move_off(g, &parts, &place),
        Dir::Backward => move_onto(g, &parts, &place),
    }
}

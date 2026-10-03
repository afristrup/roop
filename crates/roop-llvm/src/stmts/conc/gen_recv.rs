use crate::{CodegenError, Dir, FnGen, move_in, move_out};
use roop_syntax::Place;

/// Reversing a receive is a send of the value back out of its place.
pub fn gen_recv(g: &mut FnGen, chan: &str, target: &Place, dir: Dir) -> Result<(), CodegenError> {
    match dir {
        Dir::Forward => move_in(g, chan, target),
        Dir::Backward => move_out(g, chan, target),
    }
}

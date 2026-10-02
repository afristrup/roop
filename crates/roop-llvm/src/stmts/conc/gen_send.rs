use crate::{CodegenError, Dir, FnGen, move_in, move_out};
use roop_syntax::Place;

/// Reversing a send is a receive of the same message back into its place.
pub fn gen_send(g: &mut FnGen, chan: &str, source: &Place, dir: Dir) -> Result<(), CodegenError> {
    match dir {
        Dir::Forward => move_out(g, chan, source),
        Dir::Backward => move_in(g, chan, source),
    }
}

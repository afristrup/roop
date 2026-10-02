use super::{CodegenError, Dir, FnGen, gen_block, gen_place};
use roop_syntax::{Block, Place};

/// A borrow is an alias for the source address, so nothing is copied.
pub fn gen_borrow(
    g: &mut FnGen,
    name: &str,
    source: &Place,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    let slot = gen_place(g, source)?;
    g.vars.push((name.into(), slot));
    let result = gen_block(g, body, dir);
    g.vars.pop();
    result
}

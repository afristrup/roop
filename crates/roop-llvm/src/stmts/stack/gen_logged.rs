use crate::{CodegenError, Dir, FnGen, gen_block};
use roop_syntax::{Block, Place};

/// Destroying updates inside push what they destroy on `history`.
pub fn gen_logged(
    g: &mut FnGen,
    history: &Place,
    body: &Block,
    dir: Dir,
) -> Result<(), CodegenError> {
    g.logged.push(history.clone());
    let result = gen_block(g, body, dir);
    g.logged.pop();
    result
}

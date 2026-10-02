use super::{CodegenError, Dir, FnGen, gen_stmt};
use roop_syntax::Block;

/// Backward execution runs the statements in reverse order.
pub fn gen_block(g: &mut FnGen, block: &Block, dir: Dir) -> Result<(), CodegenError> {
    match dir {
        Dir::Forward => block.stmts.iter().try_for_each(|s| gen_stmt(g, s, dir)),
        Dir::Backward => block.stmts.iter().rev().try_for_each(|s| gen_stmt(g, s, dir)),
    }
}

use crate::{HybridError, Lowerer, Op, roop_stmt};
use roop_syntax::Block;

/// A block forward, or backward: its statements in reverse, each inverted.
pub fn roop_block(
    l: &mut Lowerer,
    block: &Block,
    backward: bool,
    out: &mut Vec<Op>,
) -> Result<(), HybridError> {
    let mark = l.frame.len();
    let result = match backward {
        false => block
            .stmts
            .iter()
            .try_for_each(|s| roop_stmt(l, s, false, out)),
        true => block
            .stmts
            .iter()
            .rev()
            .try_for_each(|s| roop_stmt(l, s, true, out)),
    };
    l.frame.truncate(mark);
    result
}

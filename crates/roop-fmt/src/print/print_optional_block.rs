use crate::{Ctx, Doc, print_block};
use roop_syntax::Block;

/// A block that may be left out when it is empty, like an `else` or `loop`
/// with nothing in it. Comments inside it keep it.
pub fn print_optional_block(ctx: &Ctx, block: &Block, collapse: bool) -> Option<Doc> {
    let empty = block.stmts.is_empty() && !ctx.has_comment_in(block.span);
    (!empty).then(|| print_block(ctx, block, collapse))
}

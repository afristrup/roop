use crate::{Scope, infer_stmt};
use roop_syntax::Block;

pub fn infer_block(scope: &mut Scope, block: &Block) -> Block {
    Block {
        stmts: block.stmts.iter().map(|s| infer_stmt(scope, s)).collect(),
        span: block.span,
    }
}

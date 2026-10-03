use crate::{Block, Visitor, walk_stmt};

pub fn walk_block(v: &mut dyn Visitor, block: &mut Block) {
    for stmt in &mut block.stmts {
        walk_stmt(v, &mut stmt.kind);
    }
}

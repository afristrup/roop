use crate::{OnName, visit_stmt};
use roop_syntax::Block;

pub fn visit_block(block: &mut Block, on: OnName) {
    for stmt in &mut block.stmts {
        visit_stmt(&mut stmt.kind, on);
    }
}

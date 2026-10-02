use crate::{map_blocks, start_fusion, try_absorb};
use roop_syntax::{Block, Stmt};

/// Fuses runs of adjacent parallel loops, inside nested blocks first.
pub fn fuse_block(block: &Block) -> Block {
    let stmts: Vec<Stmt> = block
        .stmts
        .iter()
        .map(|s| map_blocks(s, &fuse_block))
        .collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < stmts.len() {
        match start_fusion(&stmts[i]) {
            Some(mut fused) => {
                while stmts
                    .get(i + 1)
                    .is_some_and(|next| try_absorb(&mut fused, next))
                {
                    i += 1;
                }
                out.extend(fused.prologue);
                out.push(fused.head);
                out.extend(fused.epilogue);
            }
            None => out.push(stmts[i].clone()),
        }
        i += 1;
    }
    Block { stmts: out }
}

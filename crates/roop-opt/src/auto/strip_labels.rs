use crate::blocks_mut;
use roop_syntax::{Attr, Block};
use std::convert::Infallible;

/// Removes the labels, which only the `auto` ancillas refer to.
pub fn strip_labels(block: &mut Block) {
    for stmt in &mut block.stmts {
        stmt.attrs.retain(|a| !matches!(a, Attr::Label(_)));
        let _ = blocks_mut::<Infallible>(stmt, &mut |inner| {
            strip_labels(inner);
            Ok(())
        });
    }
}

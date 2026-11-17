use crate::{HybridProgram, Layout, Signature, Stmt, lower_block};

pub fn lower(stmts: &[Stmt], sig: &Signature) -> HybridProgram {
    let layout = Layout::new(sig);
    let ops = lower_block(stmts, &layout);
    HybridProgram { layout, ops }
}

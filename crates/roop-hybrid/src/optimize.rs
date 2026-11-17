use crate::{HybridProgram, optimize_block};

pub fn optimize(program: HybridProgram) -> HybridProgram {
    HybridProgram {
        layout: program.layout,
        ops: optimize_block(program.ops),
    }
}

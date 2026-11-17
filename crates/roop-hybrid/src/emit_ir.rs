use crate::{HybridProgram, ir_block};

pub fn emit_ir(program: &HybridProgram) -> String {
    let layout = &program.layout;
    let mut out = format!(
        "qubits {:?}\nbits {:?}\nints {:?}\n",
        layout.qubits, layout.bits, layout.ints
    );
    ir_block(&program.ops, layout, 0, &mut out);
    out
}

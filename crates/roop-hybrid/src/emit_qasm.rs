use crate::{HybridProgram, qasm_block};

/// OpenQASM 3 with real-time classical control. Qubits are the register `q`
/// and bits the register `c`, in declaration order; ints keep their names
/// behind a `v_` that keeps them clear of OpenQASM's keywords and gates.
pub fn emit_qasm(program: &HybridProgram) -> String {
    let layout = &program.layout;
    let mut out = String::from("OPENQASM 3.0;\ninclude \"stdgates.inc\";\n\n");
    for (i, name) in layout.qubits.iter().enumerate() {
        out.push_str(&format!("// q[{i}] is {name}\n"));
    }
    for (i, name) in layout.bits.iter().enumerate() {
        out.push_str(&format!("// c[{i}] is {name}\n"));
    }
    if !layout.qubits.is_empty() {
        out.push_str(&format!("qubit[{}] q;\nreset q;\n", layout.qubits.len()));
    }
    if !layout.bits.is_empty() {
        let zeros = "0".repeat(layout.bits.len());
        out.push_str(&format!("bit[{}] c = \"{zeros}\";\n", layout.bits.len()));
    }
    for name in &layout.ints {
        out.push_str(&format!("int[64] v_{name} = 0;\n"));
    }
    out.push('\n');
    qasm_block(&program.ops, layout, 0, &mut 0, &mut out);
    out
}

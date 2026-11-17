use crate::{Gate, Layout, Op, gate_label, ir_cond, ir_int};

/// The IR of a block, one op a line, with qubits by name.
pub fn ir_block(ops: &[Op], layout: &Layout, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let q = |i: usize| layout.qubits[i].as_str();
    for op in ops {
        let line = match op {
            Op::Kernel(gates) => {
                let body: Vec<String> = gates
                    .iter()
                    .map(|g| match *g {
                        Gate::Cnot(c, t) => format!("cx {} {}", q(c), q(t)),
                        Gate::X(i) | Gate::H(i) => format!("{} {}", gate_label(g), q(i)),
                        Gate::Phase(i, t) => format!("phase({}/2^{}) {}", t.num, t.log, q(i)),
                    })
                    .collect();
                format!("kernel {{ {} }}", body.join("; "))
            }
            Op::Measure { bit, qubit } => format!("{} := measure {}", layout.bits[*bit], q(*qubit)),
            Op::SetBit { bit, value } => {
                format!("{} := {}", layout.bits[*bit], ir_cond(value, layout))
            }
            Op::SetInt { reg, value } => {
                format!("{} := {}", layout.ints[*reg], ir_int(value, layout))
            }
            Op::If { cond, then, els } => {
                out.push_str(&format!("{pad}if {} {{\n", ir_cond(cond, layout)));
                ir_block(then, layout, depth + 1, out);
                out.push_str(&format!("{pad}}} else {{\n"));
                ir_block(els, layout, depth + 1, out);
                "}".to_string()
            }
            Op::While { cond, body } => {
                out.push_str(&format!("{pad}while {} {{\n", ir_cond(cond, layout)));
                ir_block(body, layout, depth + 1, out);
                "}".to_string()
            }
        };
        out.push_str(&format!("{pad}{line}\n"));
    }
}

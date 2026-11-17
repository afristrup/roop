use crate::{Cond, Layout, Op, qasm_cond, qasm_gate, qasm_int};

/// The ops of a block as OpenQASM 3 at an indentation `depth`, numbering
/// kernels from `kernels` on.
pub fn qasm_block(
    ops: &[Op],
    layout: &Layout,
    depth: usize,
    kernels: &mut usize,
    out: &mut String,
) {
    let pad = "    ".repeat(depth);
    for op in ops {
        match op {
            Op::Kernel(gates) => {
                out.push_str(&format!("{pad}// kernel {kernels}\n"));
                *kernels += 1;
                for gate in gates {
                    out.push_str(&format!("{pad}{}\n", qasm_gate(gate)));
                }
            }
            Op::Measure { bit, qubit } => {
                out.push_str(&format!("{pad}c[{bit}] = measure q[{qubit}];\n"))
            }
            Op::SetBit {
                bit,
                value: Cond::Lit(v),
            } => out.push_str(&format!("{pad}c[{bit}] = {};\n", u8::from(*v))),
            Op::SetBit { bit, value } => out.push_str(&format!(
                "{pad}if ({}) {{ c[{bit}] = 1; }} else {{ c[{bit}] = 0; }}\n",
                qasm_cond(value, layout)
            )),
            Op::SetInt { reg, value } => out.push_str(&format!(
                "{pad}v_{} = {};\n",
                layout.ints[*reg],
                qasm_int(value, layout)
            )),
            Op::If { cond, then, els } => {
                out.push_str(&format!("{pad}if ({}) {{\n", qasm_cond(cond, layout)));
                qasm_block(then, layout, depth + 1, kernels, out);
                if !els.is_empty() {
                    out.push_str(&format!("{pad}}} else {{\n"));
                    qasm_block(els, layout, depth + 1, kernels, out);
                }
                out.push_str(&format!("{pad}}}\n"));
            }
            Op::While { cond, body } => {
                out.push_str(&format!("{pad}while ({}) {{\n", qasm_cond(cond, layout)));
                qasm_block(body, layout, depth + 1, kernels, out);
                out.push_str(&format!("{pad}}}\n"));
            }
        }
    }
}

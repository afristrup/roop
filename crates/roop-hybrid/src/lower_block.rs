use crate::{Kind, Layout, Op, Stmt, lower_cond, lower_gate, lower_int};

/// Statements as IR. Declarations vanish: the layout allocates everything at
/// the start in |0⟩ or 0, which the checker makes indistinguishable from
/// allocating it later. Adjacent gates share a kernel.
pub fn lower_block(stmts: &[Stmt], layout: &Layout) -> Vec<Op> {
    let mut ops = Vec::new();
    for stmt in stmts {
        let op = match stmt {
            Stmt::Skip | Stmt::Declare { .. } => continue,
            Stmt::Gate { gate, qubits, .. } => {
                let qubits: Vec<usize> = qubits
                    .iter()
                    .map(|q| layout.index(Kind::Qubit, q))
                    .collect();
                let gate = lower_gate(*gate, &qubits);
                match ops.last_mut() {
                    Some(Op::Kernel(gates)) => {
                        gates.push(gate);
                        continue;
                    }
                    _ => Op::Kernel(vec![gate]),
                }
            }
            Stmt::Assign { name, value, .. } => match layout.bits.contains(name) {
                true => Op::SetBit {
                    bit: layout.index(Kind::Bit, name),
                    value: lower_cond(value, layout),
                },
                false => Op::SetInt {
                    reg: layout.index(Kind::Int, name),
                    value: lower_int(value, layout),
                },
            },
            Stmt::Measure { bit, qubit, .. } => Op::Measure {
                bit: layout.index(Kind::Bit, bit),
                qubit: layout.index(Kind::Qubit, qubit),
            },
            Stmt::If {
                cond, then, els, ..
            } => Op::If {
                cond: lower_cond(cond, layout),
                then: lower_block(then, layout),
                els: lower_block(els, layout),
            },
            Stmt::While { cond, body, .. } => Op::While {
                cond: lower_cond(cond, layout),
                body: lower_block(body, layout),
            },
        };
        ops.push(op);
    }
    ops
}

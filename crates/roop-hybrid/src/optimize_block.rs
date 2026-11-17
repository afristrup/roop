use crate::{Gate, Op, measured, overwritten, simplify_kernel, touches};

/// Fuses kernels. Classical assignments touch no qubit and a measurement
/// touches one, so a later gate moves back past them, into the open kernel,
/// unless it acts on a qubit measured in between. Each kernel is then
/// simplified, and assignments overwritten before any read are dropped. Fewer kernels means fewer round trips between the host and the
/// device.
pub fn optimize_block(ops: Vec<Op>) -> Vec<Op> {
    let mut out = Vec::new();
    let mut kernel: Option<Vec<Gate>> = None;
    let mut deferred = Vec::new();
    let flush = |out: &mut Vec<Op>, kernel: &mut Option<Vec<Gate>>, deferred: &mut Vec<Op>| {
        if let Some(gates) = kernel.take() {
            let gates = simplify_kernel(gates);
            if !gates.is_empty() {
                out.push(Op::Kernel(gates));
            }
        }
        out.append(deferred);
    };
    for op in ops {
        match op {
            Op::Kernel(gates) => {
                let blocked = measured(&deferred);
                let split = gates
                    .iter()
                    .position(|g| touches(g, &blocked))
                    .unwrap_or(gates.len());
                kernel.get_or_insert_with(Vec::new).extend(&gates[..split]);
                if split < gates.len() {
                    flush(&mut out, &mut kernel, &mut deferred);
                    kernel = Some(gates[split..].to_vec());
                }
            }
            Op::SetBit { .. } | Op::SetInt { .. } | Op::Measure { .. } if kernel.is_some() => {
                deferred.push(op)
            }
            Op::If { cond, then, els } => {
                flush(&mut out, &mut kernel, &mut deferred);
                let (then, els) = (optimize_block(then), optimize_block(els));
                if !then.is_empty() || !els.is_empty() {
                    out.push(Op::If { cond, then, els });
                }
            }
            Op::While { cond, body } => {
                flush(&mut out, &mut kernel, &mut deferred);
                out.push(Op::While {
                    cond,
                    body: optimize_block(body),
                });
            }
            other => {
                flush(&mut out, &mut kernel, &mut deferred);
                out.push(other);
            }
        }
    }
    flush(&mut out, &mut kernel, &mut deferred);
    (0..out.len())
        .filter(|&i| !overwritten(&out[i..]))
        .map(|i| out[i].clone())
        .collect()
}

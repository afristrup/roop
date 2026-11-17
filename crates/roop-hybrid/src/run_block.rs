use crate::{
    CqState, HybridError, Op, Run, apply_gate, eval_cond, eval_int, gate_label, project, run_while,
    split,
};

/// The denotational semantics of HQbricks∞ (Fig. 3 of Issa et al.) on
/// cq-states, op by op.
pub fn run_block(ops: &[Op], mut state: CqState, run: &mut Run) -> Result<CqState, HybridError> {
    for op in ops {
        let mass = state.trace();
        state = match op {
            Op::Kernel(gates) => {
                run.costs.kernels += mass;
                for gate in gates {
                    *run.costs.gates.entry(gate_label(gate)).or_default() += mass;
                }
                for rho in state.parts.values_mut() {
                    gates.iter().for_each(|g| apply_gate(rho, g));
                }
                state
            }
            Op::Measure { bit, qubit } => {
                run.costs.measurements += mass;
                let mut out = CqState::default();
                for (store, rho) in state.parts {
                    for value in [false, true] {
                        let part = project(&rho, *qubit, value);
                        if part.trace() > 0.0 {
                            let mut store = store.clone();
                            store.bits[*bit] = value;
                            out.insert(store, part);
                        }
                    }
                }
                out
            }
            Op::SetBit { bit, value } => {
                let mut out = CqState::default();
                for (mut store, rho) in state.parts {
                    store.bits[*bit] = eval_cond(value, &store)?;
                    out.insert(store, rho);
                }
                out
            }
            Op::SetInt { reg, value } => {
                let mut out = CqState::default();
                for (mut store, rho) in state.parts {
                    store.ints[*reg] = eval_int(value, &store)?;
                    out.insert(store, rho);
                }
                out
            }
            Op::If { cond, then, els } => {
                let (yes, no) = split(state, cond)?;
                let mut out = run_block(then, yes, run)?;
                out.merge(run_block(els, no, run)?);
                out
            }
            Op::While { cond, body } => run_while(cond, body, state, run)?,
        };
    }
    Ok(state)
}

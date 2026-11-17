use crate::{
    Costs, CqState, Density, HybridError, HybridProgram, Limits, Outcome, Run, Store, run_block,
};

/// Runs a program exactly from all-zero memory.
pub fn execute(program: &HybridProgram, limits: &Limits) -> Result<Outcome, HybridError> {
    let layout = &program.layout;
    if layout.qubits.len() > limits.qubits {
        return Err(HybridError::TooManyQubits {
            qubits: layout.qubits.len(),
            limit: limits.qubits,
        });
    }
    let store = Store {
        bits: vec![false; layout.bits.len()],
        ints: vec![0; layout.ints.len()],
    };
    let mut state = CqState::default();
    state.insert(store, Density::ground(layout.qubits.len()));
    let mut run = Run {
        limits,
        costs: Costs::default(),
        truncated: 0.0,
    };
    let state = run_block(&program.ops, state, &mut run)?;
    Ok(Outcome {
        state,
        truncated: run.truncated,
        costs: run.costs,
    })
}

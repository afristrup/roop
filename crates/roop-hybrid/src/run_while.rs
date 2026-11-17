use crate::{Cond, CqState, HybridError, Op, Run, run_block, split};

/// ⟦while b do p⟧ρ = Σ_n F_¬b (⟦p⟧ ∘ F_b)^n ρ, summed until the mass left in
/// the loop is negligible or the iteration limit is reached. What is left is
/// counted as truncated: it has not terminated yet.
pub fn run_while(
    cond: &Cond,
    body: &[Op],
    state: CqState,
    run: &mut Run,
) -> Result<CqState, HybridError> {
    let mut out = CqState::default();
    let mut pending = state;
    for round in 0.. {
        let (go, stop) = split(pending, cond)?;
        out.merge(stop);
        let mass = go.trace();
        if mass <= run.limits.epsilon || round == run.limits.iterations {
            run.truncated += mass;
            break;
        }
        run.costs.iterations += mass;
        pending = run_block(body, go, run)?;
        if pending.parts.len() + out.parts.len() > run.limits.stores {
            return Err(HybridError::TooManyStores(run.limits.stores));
        }
    }
    Ok(out)
}

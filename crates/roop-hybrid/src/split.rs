use crate::{Cond, CqState, HybridError, eval_cond};

/// The filters F_b and F_¬b: the parts of `state` where `cond` holds, and the
/// rest.
pub fn split(state: CqState, cond: &Cond) -> Result<(CqState, CqState), HybridError> {
    let (mut yes, mut no) = (CqState::default(), CqState::default());
    for (store, rho) in state.parts {
        match eval_cond(cond, &store)? {
            true => yes.insert(store, rho),
            false => no.insert(store, rho),
        }
    }
    Ok((yes, no))
}

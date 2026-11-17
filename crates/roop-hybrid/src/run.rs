use crate::{Costs, Limits};

/// What the interpreter carries through a run besides the state.
pub struct Run<'a> {
    pub limits: &'a Limits,
    pub costs: Costs,
    pub truncated: f64,
}

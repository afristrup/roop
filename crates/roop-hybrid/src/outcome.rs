use crate::{Costs, CqState};

/// The final cq-state of the runs that terminated, the mass of the runs that
/// had not when the interpreter stopped unrolling, and the expected costs.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub state: CqState,
    pub truncated: f64,
    pub costs: Costs,
}

use crate::{IterationSpace, Slot, Value};

/// A counted loop after its bounds were evaluated and its entry asserted.
pub struct ParallelLoop {
    pub var: String,
    pub slot: Slot,
    pub space: IterationSpace,
    pub step: i64,
    /// Where the induction variable ends: `hi` forward, `lo` backward.
    pub end: Value,
}

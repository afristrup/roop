use crate::quantum::Device;
use crate::world::refuse;

/// Where the qubit behind `handle` sits in the state vector.
pub fn position(d: &Device, handle: i64) -> usize {
    d.handles
        .iter()
        .position(|&h| h == handle)
        .unwrap_or_else(|| refuse("a gate was given a place that holds no live qubit"))
}

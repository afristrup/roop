use crate::quantum::Amp;
use std::sync::{Mutex, MutexGuard};

/// The most qubits alive at once: the state vector has 2^n amplitudes.
pub const MAX_QUBITS: usize = 24;

/// The simulated quantum device: a state vector over the live qubits, where
/// the qubit at position p is bit p of a basis index, and the handle each
/// position belongs to. Handle 0 is never given out, so a qubit place that is
/// still zero holds no qubit.
pub struct Device {
    pub amps: Vec<Amp>,
    pub handles: Vec<i64>,
    pub next: i64,
    pub rng: u64,
}

static DEVICE: Mutex<Device> = Mutex::new(Device {
    amps: Vec::new(),
    handles: Vec::new(),
    next: 1,
    rng: 0,
});

pub fn device() -> MutexGuard<'static, Device> {
    let mut d = DEVICE.lock().unwrap_or_else(|e| e.into_inner());
    if d.amps.is_empty() {
        d.amps.push(Amp { re: 1.0, im: 0.0 });
    }
    d
}

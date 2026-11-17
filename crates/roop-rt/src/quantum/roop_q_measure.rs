use crate::quantum::{Amp, device, position};
use crate::world::{refuse, require_zero};

/// Measures `q` in the computational basis into the zero bool `c`, and
/// collapses the state. Irreversible: there is no inverse. The outcomes are
/// drawn from a fixed seed, or from `ROOP_QUANTUM_SEED`, so a run repeats.
///
/// # Safety
/// Both pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_q_measure(q: *const i64, c: *mut u8) {
    unsafe { require_zero(c, 1, "a measurement result") };
    let mut d = device();
    let p = position(&d, unsafe { *q });
    let bit = 1usize << p;
    let one: f64 = (0..d.amps.len())
        .filter(|i| i & bit != 0)
        .map(|i| d.amps[i].norm())
        .sum();
    if d.rng == 0 {
        let seed = std::env::var("ROOP_QUANTUM_SEED")
            .ok()
            .and_then(|s| s.parse().ok());
        d.rng = seed.filter(|&s| s != 0).unwrap_or(0x9e37_79b9_7f4a_7c15);
    }
    d.rng ^= d.rng << 13;
    d.rng ^= d.rng >> 7;
    d.rng ^= d.rng << 17;
    let draw = (d.rng >> 11) as f64 / (1u64 << 53) as f64;
    let outcome = draw < one;
    let kept = if outcome { one } else { 1.0 - one };
    if kept <= 0.0 {
        refuse("a measurement had no outcome of positive probability");
    }
    let scale = 1.0 / kept.sqrt();
    for (i, a) in d.amps.iter_mut().enumerate() {
        *a = match (i & bit != 0) == outcome {
            true => Amp {
                re: a.re * scale,
                im: a.im * scale,
            },
            false => Amp::default(),
        };
    }
    unsafe { *c = u8::from(outcome) };
}

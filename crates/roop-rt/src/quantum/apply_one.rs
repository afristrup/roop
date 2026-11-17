use crate::quantum::{Amp, Device};

/// Applies the 2x2 unitary `m` to the qubit at position `p`.
pub fn apply_one(d: &mut Device, p: usize, m: [[Amp; 2]; 2]) {
    let bit = 1usize << p;
    for i in (0..d.amps.len()).filter(|i| i & bit == 0) {
        let (a, b) = (d.amps[i], d.amps[i | bit]);
        let row = |r: [Amp; 2]| {
            let (x, y) = (r[0] * a, r[1] * b);
            Amp {
                re: x.re + y.re,
                im: x.im + y.im,
            }
        };
        d.amps[i] = row(m[0]);
        d.amps[i | bit] = row(m[1]);
    }
}

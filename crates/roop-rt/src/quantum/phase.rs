use crate::quantum::Amp;

/// diag(1, e^{iθ}).
pub fn phase(theta: f64) -> [[Amp; 2]; 2] {
    let one = Amp { re: 1.0, im: 0.0 };
    let zero = Amp::default();
    [
        [one, zero],
        [
            zero,
            Amp {
                re: theta.cos(),
                im: theta.sin(),
            },
        ],
    ]
}

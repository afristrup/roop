use crate::{Complex, Density};

/// P ρ P for P the projector on `qubit` = `value`. The measurement of
/// HQbricks∞ keeps the qubit, collapsed.
pub fn project(rho: &Density, qubit: usize, value: bool) -> Density {
    let dim = rho.dim();
    let keep = |i: usize| ((i >> qubit) & 1 == 1) == value;
    let mut out = rho.clone();
    for i in 0..dim {
        for j in 0..dim {
            if !keep(i) || !keep(j) {
                out.entries[i * dim + j] = Complex::ZERO;
            }
        }
    }
    out
}

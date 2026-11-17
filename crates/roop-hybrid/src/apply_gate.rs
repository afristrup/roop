use crate::{Complex, Density, Gate};

/// ρ ↦ UρU†. Permutations and phases act entry by entry; H mixes pairs of
/// rows and then pairs of columns.
pub fn apply_gate(rho: &mut Density, gate: &Gate) {
    let dim = rho.dim();
    let bit = |i: usize, q: usize| (i >> q) & 1 == 1;
    match *gate {
        Gate::X(_) | Gate::Cnot(..) => {
            let perm = |i: usize| match *gate {
                Gate::X(q) => i ^ (1 << q),
                Gate::Cnot(c, t) if bit(i, c) => i ^ (1 << t),
                _ => i,
            };
            let old = rho.entries.clone();
            for i in 0..dim {
                for j in 0..dim {
                    rho.entries[i * dim + j] = old[perm(i) * dim + perm(j)];
                }
            }
        }
        Gate::Phase(q, t) => {
            let (up, down) = (Complex::polar(t.radians()), Complex::polar(-t.radians()));
            for i in 0..dim {
                for j in 0..dim {
                    let e = &mut rho.entries[i * dim + j];
                    match (bit(i, q), bit(j, q)) {
                        (true, false) => *e = *e * up,
                        (false, true) => *e = *e * down,
                        _ => {}
                    }
                }
            }
        }
        Gate::H(q) => {
            let s = std::f64::consts::FRAC_1_SQRT_2;
            let m = 1 << q;
            for i in (0..dim).filter(|i| i & m == 0) {
                for j in 0..dim {
                    let (a, b) = (rho.entries[i * dim + j], rho.entries[(i | m) * dim + j]);
                    rho.entries[i * dim + j] = (a + b).scale(s);
                    rho.entries[(i | m) * dim + j] = (a - b).scale(s);
                }
            }
            for i in 0..dim {
                for j in (0..dim).filter(|j| j & m == 0) {
                    let (a, b) = (rho.entries[i * dim + j], rho.entries[i * dim + (j | m)]);
                    rho.entries[i * dim + j] = (a + b).scale(s);
                    rho.entries[i * dim + (j | m)] = (a - b).scale(s);
                }
            }
        }
    }
}

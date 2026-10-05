mod support;

use support::holds;

const N: usize = 64;
const M: usize = 32;

/// Weights on the Q12 grid with entries of standard deviation `scale`, from a fixed
/// generator, as an M by N matrix when `rows` is M.
fn weights(seed: u64, rows: usize, cols: usize, scale: f64) -> Vec<Vec<i64>> {
    let mut state = seed;
    let mut uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..rows)
        .map(|_| {
            (0..cols)
                .map(|_| {
                    let (u, v) = (uniform().max(1e-12), uniform());
                    let normal = (-2.0 * u.ln()).sqrt() * (6.283185307 * v).cos();
                    (normal * scale * 4096.0).round() as i64
                })
                .collect()
        })
        .collect()
}

fn assign(name: &str, w: &[Vec<i64>]) -> String {
    let cell = |(i, row): (usize, &Vec<i64>)| {
        row.iter()
            .enumerate()
            .map(|(j, v)| format!("    {name}[{i}][{j}] += {v};\n"))
            .collect::<String>()
    };
    w.iter().enumerate().map(cell).collect()
}

/// The spectral norm in Q12, from the power iteration on W^T W.
fn spectral_norm(w: &[Vec<i64>]) -> f64 {
    let cols = w[0].len();
    let mut v = vec![1.0; cols];
    let mut norm = 0.0;
    for _ in 0..2000 {
        let wv: Vec<f64> = w
            .iter()
            .map(|row| row.iter().zip(&v).map(|(a, b)| *a as f64 * b).sum())
            .collect();
        let next: Vec<f64> = (0..cols)
            .map(|c| w.iter().zip(&wv).map(|(row, x)| row[c] as f64 * x).sum())
            .collect();
        norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
        v = next.iter().map(|x| x / norm).collect();
    }
    norm.sqrt()
}

fn setup() -> (String, Vec<Vec<i64>>, Vec<Vec<i64>>) {
    let (w1, w2) = (weights(1, M, N, 0.2), weights(2, N, M, 0.2));
    let text = format!(
        "    ancilla w1: [[i64; {N}]; {M}] = 0;\n    ancilla w2: [[i64; {M}]; {N}] = 0;\n{}{}",
        assign("w1", &w1),
        assign("w2", &w2)
    );
    (text, w1, w2)
}

const NORMS: &str = "ancilla h1: [[i64; 64]; 64] = 0;
    ancilla h2: [[i64; 32]; 32] = 0;
    ancilla a: i64 = 0;
    ancilla b: i64 = 0;
    call dominant<64, 32>(h1, a, w1);
    call dominant<32, 64>(h2, b, w2);";

#[test]
fn the_spectral_bound_of_a_wide_weight_is_close_to_its_norm() {
    let (setup, w1, w2) = setup();
    let (n1, n2) = (spectral_norm(&w1), spectral_norm(&w2));
    let check = format!(
        "a >= {} && a <= {} && b >= {} && b <= {}",
        (n1 * 0.99) as i64,
        (n1 * 1.05) as i64,
        (n2 * 0.99) as i64,
        (n2 * 1.05) as i64
    );
    assert!(holds("wide_norm", &setup, NORMS, &check));
}

#[test]
fn a_wide_pair_above_the_cap_is_shrunk_to_a_true_bound_under_it() {
    let (setup, w1, w2) = setup();
    let high = spectral_norm(&w1) * spectral_norm(&w2) / 4096.0;
    let factor = (3276.0 / high).sqrt();
    let (i, j) = (0..M)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .max_by_key(|&(i, j)| w1[i][j])
        .unwrap();
    let call = format!("call project_contraction<{N}, {M}>(w1, w2, 4096, 3276);");
    let check = format!("w1[{i}][{j}] <= {}", (w1[i][j] as f64 * factor) as i64);
    assert!(holds("wide_cap", &setup, &call, &check));
}

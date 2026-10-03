mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

/// Row count, column count and inner length: ragged against the 16 by 32 tiles
/// of the matrix kernel, past its 256-deep blocks, and big enough for threads.
const SHAPES: [(usize, usize, usize); 4] =
    [(17, 17, 17), (37, 53, 29), (130, 70, 300), (300, 500, 520)];

fn std_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/std")
}

fn program() -> String {
    let mut text = String::from("use std::blas::level3::dgemm;\n");
    for (m, n, k) in SHAPES {
        text.push_str(&format!(
            "fn mm{m}(c: &mut [[f64; {n}]; {m}], a: &[[f64; {k}]; {m}], b: &[[f64; {n}]; {k}], x: &f64) {{ call dgemm<{m}, {n}, {k}>(c, a, b, x); }}\n"
        ));
    }
    text
}

fn driver() -> String {
    let mut text = String::from(
        r#"#include <math.h>
#include <stdlib.h>
static double fill(long i) { return (double)((i * 7919) % 211) / 64.0 - 1.5; }
static int check(void (*mm)(double*, double*, double*, double*), void (*inv)(double*, double*, double*, double*), long m, long n, long k) {
    double *a = malloc(m * k * 8), *b = malloc(k * n * 8), *c = malloc(m * n * 8);
    double *want = malloc(m * n * 8), *start = malloc(m * n * 8), x = 0.75;
    for (long i = 0; i < m * k; i++) a[i] = fill(i);
    for (long i = 0; i < k * n; i++) b[i] = fill(i + 5);
    for (long i = 0; i < m * n; i++) start[i] = c[i] = want[i] = fill(i + 11);
    for (long i = 0; i < m; i++)
        for (long l = 0; l < k; l++)
            for (long j = 0; j < n; j++) want[i * n + j] += x * a[i * k + l] * b[l * n + j];
    mm(c, a, b, &x);
    double forward = 0, back = 0;
    for (long i = 0; i < m * n; i++) forward = fmax(forward, fabs(c[i] - want[i]));
    inv(c, a, b, &x);
    for (long i = 0; i < m * n; i++) back = fmax(back, fabs(c[i] - start[i]));
    return forward < 1e-9 && back < 1e-9 ? 0 : 1;
}
"#,
    );
    for (m, _, _) in SHAPES {
        text.push_str(&format!(
            "void mm{m}(double*, double*, double*, double*);\nvoid mm{m}_inv(double*, double*, double*, double*);\n"
        ));
    }
    text.push_str("int main(void) {\n");
    for (index, (m, n, k)) in SHAPES.into_iter().enumerate() {
        text.push_str(&format!(
            "    if (check(mm{m}, mm{m}_inv, {m}, {n}, {k})) return {};\n",
            index + 1
        ));
    }
    text.push_str("    return 0;\n}\n");
    text
}

fn run_with(config_tail: &str) {
    let config = format!(
        "[modules]\nstd = \"{}\"\n{config_tail}",
        std_dir().display()
    );
    let dir = project("dgemm-kernel", &config, &program());
    std::fs::write(dir.join("main.c"), driver()).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let code = Command::new(dir.join("prog")).status().unwrap().code();
    assert_eq!(
        code,
        Some(0),
        "shape #{code:?} disagreed with the plain loops"
    );
}

#[test]
fn the_matrix_kernel_matches_the_plain_loops_and_reverses() {
    run_with("");
}

#[test]
fn the_loops_agree_when_the_matrix_unit_is_switched_off() {
    run_with("\n[parallel]\nsme = false\n");
}

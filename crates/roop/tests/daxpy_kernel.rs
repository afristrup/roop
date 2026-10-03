mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

/// Lengths that are ragged against the kernel's blocks, and one long enough
/// for it to use threads.
const LENGTHS: [usize; 4] = [2048, 5000, 70001, 600001];

fn std_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/std")
}

fn program() -> String {
    let mut text = String::from("use std::blas::level1::daxpy;\n");
    for n in LENGTHS {
        text.push_str(&format!(
            "fn ax{n}(y: &mut [f64; {n}], x: &[f64; {n}], a: &f64) {{ call daxpy<{n}>(y, x, a); }}\n"
        ));
    }
    text
}

fn driver() -> String {
    let mut text = String::from(
        r#"#include <math.h>
#include <stdlib.h>
static double fill(long i) { return (double)((i * 7919) % 211) / 64.0 - 1.5; }
static int check(void (*ax)(double*, double*, double*), void (*inv)(double*, double*, double*), long n) {
    double *x = malloc(n * 8), *y = malloc(n * 8), *want = malloc(n * 8), *start = malloc(n * 8), a = 0.75;
    for (long i = 0; i < n; i++) { x[i] = fill(i); start[i] = y[i] = want[i] = fill(i + 5); }
    for (long i = 0; i < n; i++) want[i] += a * x[i];
    ax(y, x, &a);
    double forward = 0, back = 0;
    for (long i = 0; i < n; i++) forward = fmax(forward, fabs(y[i] - want[i]));
    inv(y, x, &a);
    for (long i = 0; i < n; i++) back = fmax(back, fabs(y[i] - start[i]));
    return forward < 1e-12 && back < 1e-12 ? 0 : 1;
}
"#,
    );
    for n in LENGTHS {
        text.push_str(&format!(
            "void ax{n}(double*, double*, double*);\nvoid ax{n}_inv(double*, double*, double*);\n"
        ));
    }
    text.push_str("int main(void) {\n");
    for (index, n) in LENGTHS.into_iter().enumerate() {
        text.push_str(&format!(
            "    if (check(ax{n}, ax{n}_inv, {n})) return {};\n",
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
    let dir = project("daxpy-kernel", &config, &program());
    std::fs::write(dir.join("main.c"), driver()).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let code = Command::new(dir.join("prog")).status().unwrap().code();
    assert_eq!(code, Some(0), "length #{code:?} disagreed with the loop");
}

#[test]
fn the_axpy_kernel_matches_the_plain_loop_and_reverses() {
    run_with("");
}

#[test]
fn the_loops_agree_when_the_matrix_unit_is_switched_off() {
    run_with("\n[parallel]\nsme = false\n");
}

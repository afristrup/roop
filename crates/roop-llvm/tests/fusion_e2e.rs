mod support;

use roop_llvm::{Compiled, Options, compile_all};
use roop_opt::fuse_parallel;
use roop_syntax::parse;

const N: usize = 1000;

fn program(target: &str) -> String {
    format!(
        "rev fn two(a: &mut [i64; {N}], b: &mut [i64; {N}], i: &mut i64, j: &mut i64, k: &i64) {{
            #[parallel({target})] from i == 0 {{ a[i] += b[i] * k; }} loop {{ i += 1; }} until i == {last};
            #[parallel({target})] from j == 0 {{ b[j] += a[j]; }} loop {{ j += 1; }} until j == {last};
        }}",
        last = N - 1
    )
}

fn build(target: &str, fuse: bool) -> Compiled {
    let program = parse(&program(target)).unwrap();
    roop_check::check(&program).unwrap();
    let program = if fuse {
        fuse_parallel(&program)
    } else {
        program
    };
    roop_check::check(&program).unwrap();
    compile_all(&program, &Options::default()).unwrap()
}

/// a[m] = m, b[m] = 2m, k = 3. After `two`: a = 7m, b = 9m, i = j = N-1.
/// Running `two_inv` must restore everything, induction variables included.
const HARNESS: &str = r#"
#include <stdint.h>
#define N 1000
void two(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
void two_inv(int64_t*, int64_t*, int64_t*, int64_t*, int64_t*);
static int64_t a[N], b[N];
int main(void) {
    int64_t i = 0, j = 0, k = 3;
    for (int64_t m = 0; m < N; m++) { a[m] = m; b[m] = 2 * m; }
    two(a, b, &i, &j, &k);
    for (int64_t m = 0; m < N; m++) {
        if (a[m] != 7 * m || b[m] != 9 * m) return 1;
    }
    if (i != N - 1 || j != N - 1) return 2;
    two_inv(a, b, &i, &j, &k);
    for (int64_t m = 0; m < N; m++) {
        if (a[m] != m || b[m] != 2 * m) return 3;
    }
    return (i != 0 || j != 0) ? 4 : 0;
}
"#;

fn run_cpu(fuse: bool) {
    let out = build("cpu", fuse);
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("main.c".to_string(), HARNESS),
    ];
    if let Some(code) = support::run_native_files(&files) {
        assert_eq!(code, Some(0), "fuse = {fuse}");
    }
}

fn metal_available() -> bool {
    cfg!(target_os = "macos")
        && std::process::Command::new("xcrun")
            .args(["-sdk", "macosx", "--find", "metal"])
            .output()
            .is_ok_and(|o| o.status.success())
}

fn run_metal(fuse: bool) {
    if !metal_available() {
        return;
    }
    let out = build("metal", fuse);
    let blob = support::metallib_blob_module(&support::metallib(&out.air.unwrap()).unwrap());
    let files = [
        ("m0.ll".to_string(), out.host.as_str()),
        ("m1.ll".to_string(), blob.as_str()),
        ("main.c".to_string(), HARNESS),
    ];
    assert_eq!(
        support::run_native_files(&files).unwrap(),
        Some(0),
        "fuse = {fuse}"
    );
}

#[test]
fn cpu_results_match_with_and_without_fusion() {
    run_cpu(false);
    run_cpu(true);
}

#[test]
fn gpu_results_match_with_and_without_fusion() {
    run_metal(false);
    run_metal(true);
}

#[test]
fn fusion_halves_the_kernel_count() {
    let kernels = |fuse| {
        build("metal", fuse)
            .air
            .unwrap()
            .matches("define void @")
            .count()
    };
    assert_eq!(kernels(false), 4); // two loops, forward and inverse
    assert_eq!(kernels(true), 2);
}

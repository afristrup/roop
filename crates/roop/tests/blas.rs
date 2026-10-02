mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{project, roop, stderr};

const PROGRAM: &str = "
use std::blas::level1::iscal;
use std::blas::level1::icopy;
use std::blas::level1::idot;
use std::blas::level2::igemv;
use std::blas::level2::itrmv;
use std::blas::level2::itrsv;
use std::blas::level2::dgemv;
use std::blas::level3::igemm;

fn scale(x: &mut [i64; 4], k: &i64) { call iscal<4>(x, k); }
fn copy(y: &mut [i64; 3], x: &[i64; 3], h: &mut Stack<i64, 3>) { call icopy<3>(y, x, h); }
fn dot(r: &mut i64, x: &[i64; 3], y: &[i64; 3]) { call idot<3>(r, x, y); }
fn mv(y: &mut [i64; 2], a: &[[i64; 3]; 2], x: &[i64; 3], k: &i64) { call igemv<2, 3>(y, a, x, k); }
fn fmv(y: &mut [f64; 2], a: &[[f64; 3]; 2], x: &[f64; 3], k: &f64) { call dgemv<2, 3>(y, a, x, k); }
fn mm(c: &mut [[i64; 2]; 2], a: &[[i64; 3]; 2], b: &[[i64; 2]; 3], k: &i64) { call igemm<2, 2, 3>(c, a, b, k); }
fn tri(x: &mut [i64; 3], a: &[[i64; 3]; 3]) { call itrmv<3>(x, a); }
fn solve(x: &mut [i64; 3], a: &[[i64; 3]; 3]) { call itrsv<3>(x, a); }
";

const MAIN: &str = r#"
#include <stdint.h>
typedef struct { int64_t len; int64_t data[3]; } Stack3;
void scale(int64_t*, int64_t*);       void scale_inv(int64_t*, int64_t*);
void copy(int64_t*, int64_t*, Stack3*); void copy_inv(int64_t*, int64_t*, Stack3*);
void dot(int64_t*, int64_t*, int64_t*);
void mv(int64_t*, int64_t*, int64_t*, int64_t*);   void mv_inv(int64_t*, int64_t*, int64_t*, int64_t*);
void fmv(double*, double*, double*, double*);      void fmv_inv(double*, double*, double*, double*);
void mm(int64_t*, int64_t*, int64_t*, int64_t*);   void mm_inv(int64_t*, int64_t*, int64_t*, int64_t*);
void tri(int64_t*, int64_t*);         void tri_inv(int64_t*, int64_t*);
void solve(int64_t*, int64_t*);       void solve_inv(int64_t*, int64_t*);

int main(void) {
    int64_t x4[4] = {1, 2, 3, 4}, three = 3;
    scale(x4, &three);
    if (x4[0] != 3 || x4[1] != 6 || x4[2] != 9 || x4[3] != 12) return 1;
    scale_inv(x4, &three);
    if (x4[0] != 1 || x4[3] != 4) return 2;

    int64_t y3[3] = {5, 6, 7}, x3[3] = {1, 2, 3};
    Stack3 h = {0};
    copy(y3, x3, &h);
    if (y3[0] != 1 || y3[2] != 3 || h.len != 3 || h.data[0] != 5 || h.data[2] != 7) return 3;
    copy_inv(y3, x3, &h);
    if (y3[0] != 5 || y3[2] != 7 || h.len != 0 || h.data[0] != 0) return 4;

    int64_t r = 0, a3[3] = {1, 2, 3}, b3[3] = {4, 5, 6};
    dot(&r, a3, b3);
    if (r != 32) return 5;

    int64_t a23[6] = {1, 2, 3, 4, 5, 6}, ones[3] = {1, 1, 1}, y2[2] = {0, 0}, two = 2;
    mv(y2, a23, ones, &two);
    if (y2[0] != 12 || y2[1] != 30) return 6;
    mv_inv(y2, a23, ones, &two);
    if (y2[0] != 0 || y2[1] != 0) return 7;

    double fa[6] = {1, 2, 3, 4, 5, 6}, fx[3] = {1, 0.5, 0.25}, fy[2] = {0, 0}, half = 0.5;
    fmv(fy, fa, fx, &half);
    if (fy[0] != 0.5 * (1 + 1 + 0.75) || fy[1] != 0.5 * (4 + 2.5 + 1.5)) return 8;

    int64_t b32[6] = {1, 0, 0, 1, 1, 1}, c[4] = {0, 0, 0, 0}, one = 1;
    mm(c, a23, b32, &one);
    if (c[0] != 4 || c[1] != 5 || c[2] != 10 || c[3] != 11) return 9;
    mm_inv(c, a23, b32, &one);
    if (c[0] != 0 || c[3] != 0) return 10;

    int64_t l[9] = {9, 9, 9, 2, 9, 9, 3, 4, 9}, v[3] = {1, 2, 3};
    tri(v, l);
    if (v[0] != 1 || v[1] != 4 || v[2] != 14) return 11;
    solve(v, l);
    if (v[0] != 1 || v[1] != 2 || v[2] != 3) return 12;
    tri(v, l);
    tri_inv(v, l);
    if (v[0] != 1 || v[1] != 2 || v[2] != 3) return 13;
    return 0;
}
"#;

fn std_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../roop/std")
}

#[test]
fn the_blas_routines_compute_the_right_values_and_reverse_exactly() {
    let config = format!("[modules]\nstd = \"{}\"\n", std_dir().display());
    let dir = project("blas-run", &config, PROGRAM);
    std::fs::write(dir.join("main.c"), MAIN).unwrap();
    let out = roop(
        &dir,
        &["build", "prog.roop", "--link", "main.c", "-o", "prog"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let code = Command::new(dir.join("prog")).status().unwrap().code();
    assert_eq!(code, Some(0), "check #{code:?} in the C driver failed");
}

#[test]
fn lean_proves_the_blas_routines_reversible() {
    let config = format!("[modules]\nstd = \"{}\"\n", std_dir().display());
    let dir = project("blas-lean", &config, PROGRAM);
    let out = roop(&dir, &["lean", "prog.roop", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report = String::from_utf8_lossy(&out.stdout).into_owned();
    for routine in ["iscal", "icopy", "idot", "igemv", "igemm", "itrmv", "itrsv"] {
        assert!(report.contains(routine), "{routine} missing: {report}");
    }
    assert!(report.contains("Lean accepted the file"), "{report}");
}

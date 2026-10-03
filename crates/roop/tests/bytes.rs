mod support;

use support::{project, roop, stderr};

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn passes(name: &str, src: &str) {
    let dir = project(name, "", src);
    let out = roop(&dir, &["test"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains(" 0 failed"), "{}", stdout(&out));
}

#[test]
fn a_byte_wraps_around_and_comes_back() {
    passes(
        "u8-wrap",
        "test wraps { a: u8; a += 250; a += 10; expect a == 4; }
         test wraps_down { a: u8; a -= 1; expect a == 255; }",
    );
}

#[test]
fn bytes_compare_and_divide_without_sign() {
    passes(
        "u8-unsigned",
        "test larger { a: u8, b: u8; a += 200; b += 100; expect a > b && b < a && a >= a; }
         test divides { a: u8, q: u8, r: u8; a += 200; q += a / 7; r += a % 7; expect q == 28 && r == 4; }",
    );
}

#[test]
fn bytes_xor_and_the_literal_takes_the_byte_type() {
    passes(
        "u8-xor",
        "test xors { a: u8; a += 240; a ^= 15; expect a == 255; a ^= 255; expect a == 0; }",
    );
}

#[test]
fn casts_convert_between_the_number_types() {
    passes(
        "casts",
        "test widens { a: u8, n: i64; a += 200; n += a as i64; expect n == 200; }
         test truncates { n: i64, c: u8; n += 300; c += n as u8; expect c == 44; }
         test negative_truncates { n: i64, c: u8; n -= 1; c += n as u8; expect c == 255; }
         test to_float { n: i64, x: f64; n += 3; x += n as f64; expect x == 3.0; }
         test from_float { x: f64, n: i64; x += 2.75; n += x as i64; expect n == 2; }
         test saturates { x: f64, n: i64, c: u8; x += 100000000000000000000000000000.0; n += x as i64; c += x as u8; expect n == 9223372036854775807 && c == 255; }",
    );
}

#[test]
fn a_bool_casts_to_a_number() {
    passes(
        "bool-cast",
        "test counts { n: i64, a: i64; a += 3; n += (a > 2) as i64; n += (a > 5) as i64; expect n == 1; }",
    );
}

#[test]
fn text_is_an_array_of_bytes() {
    passes(
        "u8-array",
        "test fills { t: [u8; 4]; t[0] += b'h'; t[1] += b'i'; expect t[0] == 104 && t[1] == 105 && t[2] == 0; }",
    );
}

#[test]
fn a_byte_literal_out_of_range_is_refused() {
    let dir = project("u8-range", "", "test bad { a: u8; a += 300; expect true; }");
    let out = roop(&dir, &["test"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("0 to 255"), "{}", stderr(&out));
}

#[test]
fn bytes_and_numbers_do_not_mix_without_a_cast() {
    let dir = project(
        "u8-mix",
        "",
        "test bad { a: u8, n: i64; a += n; expect true; }",
    );
    let out = roop(&dir, &["test"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("expected Named"), "{}", stderr(&out));
}

#[test]
fn a_string_is_a_read_only_array() {
    let dir = project(
        "str-mut",
        "",
        "fn put(s: &mut [u8; 3]) { s[0] += 1; }\nfn f() { call put(\"abc\"); }",
    );
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
}

#[test]
fn lengths_are_inferred_from_the_arguments() {
    passes(
        "infer",
        "fn total<N>(r: &mut i64, xs: &[i64; N]) {
             ancilla i: i64 = 0;
             from i == 0 { r += xs[i]; } loop { i += 1; } until i == N - 1;
             i -= N - 1;
         }
         fn total_of<M>(r: &mut i64, ys: &[i64; M]) { call total(r, ys); }
         test infers_from_a_fixture { r: i64, xs: [i64; 3]; xs[0] += 1; xs[1] += 2; xs[2] += 3; call total(r, xs); expect r == 6; }
         test infers_from_the_callers_own_length { r: i64, xs: [i64; 2]; xs[1] += 5; call total_of(r, xs); expect r == 5; }
         test infers_through_an_ancilla { r: i64; ancilla t: [i64; 2] = 0; t[0] += 4; call total(r, t); expect r == 4; t[0] -= 4; }",
    );
}

#[test]
fn lengths_are_inferred_for_two_arrays_and_a_matrix() {
    passes(
        "infer-many",
        "fn pair<N, M>(r: &mut i64, a: &[i64; N], b: &[[i64; N]; M]) { r += N * 10 + M; }
         test two { r: i64, a: [i64; 3], b: [[i64; 3]; 2]; call pair(r, a, b); expect r == 32; }",
    );
}

#[test]
fn lengths_that_disagree_are_reported() {
    let dir = project(
        "infer-disagree",
        "",
        "fn same<N>(a: &[i64; N], b: &[i64; N]) { }
         fn f(a: &[i64; 2], b: &[i64; 3]) { call same(a, b); }",
    );
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("lengths"), "{}", stderr(&out));
}

#[test]
fn lengths_that_cannot_be_found_are_reported() {
    let dir = project(
        "infer-missing",
        "",
        "fn make<N>(a: &mut i64) { a += N; }\nfn f(a: &mut i64) { call make(a); }",
    );
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("takes 1 lengths"), "{}", stderr(&out));
}

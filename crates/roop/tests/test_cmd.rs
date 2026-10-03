mod support;

use support::{project, roop, stderr};

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const COUNTER: &str = "
fn bump(a: &mut i64, k: &i64) {
    a += k;
}

test bumps_by_k {
    a: i64, k: i64;
    k += 3;
    call bump(a, k);
    expect a == 3;
}

test bumps_back_with_uncall {
    a: i64, k: i64;
    a += 3;
    k += 3;
    uncall bump(a, k);
    expect a == 0;
}
";

#[test]
fn passing_tests_are_reported_and_succeed() {
    let dir = project("test-pass", "", COUNTER);
    let out = roop(&dir, &["test"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("test bumps_by_k ... ok"), "{text}");
    assert!(
        text.contains("test bumps_back_with_uncall ... ok"),
        "{text}"
    );
    assert!(text.contains("2 passed, 0 failed"), "{text}");
}

#[test]
fn a_failed_expectation_fails_that_test_only() {
    let src = format!("{COUNTER}\ntest wrong {{ a: i64; expect a == 1; }}\n");
    let dir = project("test-fail", "", &src);
    let out = roop(&dir, &["test"]);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(text.contains("test wrong ... FAILED"), "{text}");
    assert!(text.contains("test bumps_by_k ... ok"), "{text}");
    assert!(text.contains("2 passed, 1 failed"), "{text}");
}

#[test]
fn a_backward_run_that_does_not_restore_is_caught() {
    let src = "
test floats_do_not_come_back {
    x: f64;
    x += 0.1;
    x += 0.2;
}
";
    let dir = project("test-restore", "", src);
    let out = roop(&dir, &["test"]);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("after the backward run, fixture x is not zero"),
        "{text}"
    );
}

#[test]
fn filter_selects_tests_by_name() {
    let dir = project("test-filter", "", COUNTER);
    let out = roop(&dir, &["test", "--filter", "uncall"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("1 passed"), "{text}");
    assert!(!text.contains("bumps_by_k"), "{text}");
}

#[test]
fn a_file_without_tests_is_skipped() {
    let dir = project("test-none", "", "fn f(a: &mut i64) { a += 1; }");
    let out = roop(&dir, &["test"]);
    assert!(out.status.success());
    assert!(
        stdout(&out).contains("0 passed, 0 failed"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn build_leaves_the_tests_out() {
    let dir = project("test-build", "", COUNTER);
    let out = roop(&dir, &["build", "prog.roop", "--emit", "ir"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let ir = std::fs::read_to_string(dir.join("prog.ll")).unwrap();
    assert!(ir.contains("@bump"), "{ir}");
    assert!(!ir.contains("@bumps_by_k"), "{ir}");
}

#[test]
fn tests_check_like_any_function() {
    let src = "test bad { a: i64; a += a; }";
    let dir = project("test-check", "", src);
    let out = roop(&dir, &["test"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("roop:"), "{}", stderr(&out));
}

#[test]
fn the_library_tests_pass() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../roop");
    let out = roop(&root, &["test"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains(" 0 failed"), "{}", stdout(&out));
}

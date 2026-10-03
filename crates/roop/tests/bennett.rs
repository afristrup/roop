mod support;

use support::{project, roop, stderr};

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const SETTLE: &str = "
fn settle_to(balance: &mut i64, amount: &i64, h: &mut Stack<i64, 8>) {
    logged h { balance = amount * 2; }
}
";

#[test]
fn a_bennett_version_drops_the_history_and_keeps_the_input() {
    let src = format!(
        "{SETTLE}
bennett fn quote = settle_to;

test quote_keeps_the_input_and_the_result {{
    balance: i64, amount: i64, balance_out: i64;
    balance += 7;
    amount += 5;
    call quote(balance, amount, balance_out);
    expect balance == 7 && amount == 5 && balance_out == 10;
}}
"
    );
    let dir = project("bennett-scalar", "", &src);
    let out = roop(&dir, &["test"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn arrays_and_lengths_carry_over() {
    let src = "
fn blank<N>(x: &mut [i64; N], h: &mut Stack<i64, 4>) {
    logged h { x[0] = 0; }
}

bennett fn blanked = blank;

test blanked_copies_the_whole_array {
    x: [i64; 3], x_out: [i64; 3];
    x[0] += 4;
    x[1] += 5;
    x[2] += 6;
    call blanked<3>(x, x_out);
    expect x[0] == 4 && x[1] == 5 && x[2] == 6;
    expect x_out[0] == 0 && x_out[1] == 5 && x_out[2] == 6;
}
";
    let dir = project("bennett-array", "", src);
    let out = roop(&dir, &["test"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn a_bennett_version_is_proved_reversible_by_lean() {
    let src = format!(
        "{SETTLE}
bennett fn quote = settle_to;
fn use_it(balance: &mut i64, amount: &i64, balance_out: &mut i64) {{
    call quote(balance, amount, balance_out);
}}
"
    );
    let dir = project("bennett-lean", "", &src);
    let out = roop(&dir, &["lean", "prog.roop"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("quote"), "{}", stdout(&out));
}

#[test]
fn an_irreversible_target_has_nothing_to_uncompute_with() {
    let src = "irrev fn f(a: &mut i64) { a = 1; }\nbennett fn g = f;\nfn h(a: &mut i64, b: &mut i64) { call g(a, b); }";
    let dir = project("bennett-irrev", "", src);
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("irreversible"), "{}", stderr(&out));
}

#[test]
fn an_unknown_target_is_reported() {
    let dir = project("bennett-unknown", "", "bennett fn g = nowhere;");
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("nowhere"), "{}", stderr(&out));
}

#[test]
fn a_result_that_cannot_be_copied_is_reported() {
    let src = "enum Color { Red, Green }\nfn f(c: &mut Color) { }\nbennett fn g = f;\nfn h(c: &mut Color, d: &mut Color) { call g(c, d); }";
    let dir = project("bennett-uncopyable", "", src);
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("cannot be copied"),
        "{}",
        stderr(&out)
    );
}

mod support;

const CALLEES: &str = "
fn bump(x: &mut i64, k: &i64) { ancilla t: i64 = 0; t += k; x += t; t -= k; }
fn double(x: &mut i64, k: &i64) { ancilla t: i64 = 0; t += k; x += t; x += t; t -= k; }
";

#[test]
fn a_function_of_calls_is_proved_one_call_at_a_time() {
    let src = format!(
        "{CALLEES}
fn steps(x: &mut i64, y: &mut i64, k: &i64) {{
    call bump(x, k);
    call double(y, k);
    uncall bump(x, k);
    ancilla n: i64 = 0;
    n += k;
    call bump(y, n);
    n -= k;
}}"
    );
    let t = support::verified(&src);
    assert!(
        t.reversible.contains(&"steps".to_string()),
        "{:?}",
        t.skipped
    );
    assert!(
        t.lean.contains("Roop.bind_ancilla \u{ab}__h\u{bb}"),
        "{}",
        t.lean
    );
    assert!(
        t.lean.contains("Roop.bind_ok \u{ab}__h\u{bb}"),
        "{}",
        t.lean
    );
}

#[test]
fn a_function_with_a_loop_keeps_the_general_proof() {
    let src = format!(
        "{CALLEES}
fn looped(x: &mut i64, k: &i64) {{
    ancilla i: i64 = 0;
    from i == 0 {{ call bump(x, k); }} loop {{ i += 1; }} until i == 2;
    i -= 2;
}}"
    );
    let t = support::verified(&src);
    assert!(!t.lean.contains("Roop.bind_ancilla \u{ab}__h\u{bb}"));
}

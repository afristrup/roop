mod support;

#[test]
fn a_function_over_bytes_and_casts_is_proved_reversible() {
    let t = support::verified(
        "fn bump(c: &mut u8, n: &i64) { c += n as u8; c ^= 255; }
         fn mix(a: &mut [u8; 4], k: &u8) {
             ancilla i: i64 = 0;
             from i == 0 { a[i] += k; a[i] += b'a'; } loop { i += 1; } until i == 3;
             i -= 3;
         }",
    );
    assert_eq!(t.reversible, ["bump", "mix"]);
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn bytes_compare_and_divide_without_sign_in_the_model() {
    let t = support::verified(
        "fn pick(r: &mut i64, c: &u8) {
             if c > 100 { r += 1; } else { r += 2; } fi c > 100;
             if c / 3 == 5 { r += 10; } else { } fi c / 3 == 5;
         }",
    );
    assert_eq!(t.reversible, ["pick"]);
}

#[test]
fn a_loop_over_bytes_gets_a_termination_theorem() {
    let t = support::verified(
        "fn count(a: &mut [u8; 4]) {
             ancilla i: i64 = 0;
             from i == 0 { a[0] += 1; } loop { i += 1; } until i == 3;
             i -= 3;
         }",
    );
    assert!(t.lean.contains("loop_terminates"));
}

#[test]
fn a_string_literal_is_a_vector_of_bytes() {
    let t = support::verified(
        "fn first(r: &mut u8, s: &[u8; 3]) { r += s[0]; }
         fn use_it(r: &mut u8) { call first(r, \"abc\"); }",
    );
    assert_eq!(t.reversible, ["first", "use_it"]);
}

#[test]
fn what_the_runtime_provides_is_not_modelled_and_nor_its_callers() {
    let t = support::translation(
        "extern world fn roop_out(fd: &i64, buf: &[u8; 3], len: &i64);
         fn print(s: &[u8; 3]) { call roop_out(1, s, 3); }
         fn add(a: &mut i64) { a += 1; }",
    );
    assert_eq!(t.reversible, ["add"]);
    let skipped: Vec<_> = t.skipped.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        skipped.contains(&"roop_out") && skipped.contains(&"print"),
        "{skipped:?}"
    );
    support::lean_accepts(&t).unwrap();
}

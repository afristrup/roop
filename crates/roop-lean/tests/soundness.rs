mod support;

#[test]
fn a_branch_exit_assertion_that_decides_the_branch_is_proved_reversible() {
    // `y > 0` holds after the then branch and fails after the else branch, so
    // the inverse can tell which one ran: when a run gets through both
    // assertions, it can be undone. When it does not, the program traps.
    let t = support::verified(
        "fn f(x: &mut i64, y: &mut i64) {
            if x > 0 { y += x; } else { y -= x; } fi y > 0;
         }",
    );
    assert_eq!(t.reversible, ["f"]);
}

#[test]
fn lean_rejects_a_corrupted_inverse() {
    let mut t = support::translation("fn add(x: &mut i64, k: &i64) { x += k; }");
    assert!(support::lean_accepts(&t).is_ok());
    // Make the inverse add instead of subtract.
    t.lean = t.lean.replace(
        "(\u{ab}x\u{bb} - \u{ab}k\u{bb})",
        "(\u{ab}x\u{bb} + \u{ab}k\u{bb})",
    );
    assert!(
        support::lean_accepts(&t).is_err(),
        "a wrong inverse must not verify"
    );
}

#[test]
fn lean_rejects_a_match_whose_exit_assertions_overlap() {
    let t = support::translation(
        "fn f(s: &i64, y: &mut i64) {
            match s {
                0 => { y += 1; } assert y > 0;
                _ => { y += 2; } assert y > 0;
            }
         }",
    );
    assert!(support::lean_accepts(&t).is_err());
}

#[test]
fn ancilla_leak_theorems_hold_for_restored_ancillas() {
    let t = support::verified(
        "fn f(x: &mut i64, y: &mut i64) {
            ancilla t: i64 = 0 { t += x; y += t; t -= x; }
         }",
    );
    assert!(t.lean.contains("f_ancilla_restored"));
    assert!(t.lean.contains("f_inv_ancilla_restored"));
}

#[test]
fn lean_rejects_an_ancilla_that_is_not_restored() {
    // Irreversible code may skip restoration, but if the same shape appears in
    // a reversible function the leak theorem cannot be proved. Build the
    // faulty file by corrupting the restoring update.
    let mut t = support::translation(
        "fn f(x: &mut i64, y: &mut i64) {
            ancilla t: i64 = 0 { t += x; y += t; t -= x; }
         }",
    );
    t.lean = t.lean.replacen(
        "(\u{ab}t\u{bb} - \u{ab}x\u{bb})",
        "(\u{ab}t\u{bb} + \u{ab}x\u{bb})",
        1,
    );
    assert!(support::lean_accepts(&t).is_err());
}

mod support;

#[test]
fn lean_rejects_a_branch_whose_exit_assertion_does_not_identify_it() {
    // The roop checker accepts this: it has an exit assertion. But `y > 0`
    // holds after both branches, so the inverse cannot tell which one ran
    // (x = -1, y = 3 takes the else branch and the inverse undoes the other).
    let t = support::translation(
        "fn f(x: &mut i64, y: &mut i64) {
            if x > 0 { y += x; } else { y -= x; } fi y > 0;
         }",
    );
    assert!(t.skipped.is_empty());
    assert!(
        support::lean_accepts(&t).is_err(),
        "Lean should not prove this reversible"
    );
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

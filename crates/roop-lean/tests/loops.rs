mod support;

#[test]
fn a_counting_loop_is_proved_reversible() {
    let t = support::verified(
        "fn count(x: &mut i64, i: &mut i64, n: &i64) {
            from i == 0 { x += 2; } loop { i += 1; } until i == n;
         }",
    );
    assert_eq!(t.reversible, ["count"]);
    assert!(!t.lean.contains("sorry"));
    assert!(t.lean.contains("Thm.\u{ab}count__loop"));
}

#[test]
fn nested_loops_are_proved_from_the_inner_loop_outwards() {
    let t = support::verified(
        "fn grid(x: &mut i64, i: &mut i64, j: &mut i64, n: &i64, m: &i64) {
            from i == 0 {
                from j == 0 { x += i; } loop { j += 1; } until j == m;
                j -= m;
            } loop { i += 1; } until i == n;
         }",
    );
    assert_eq!(t.reversible, ["grid"]);
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn loops_over_arrays_with_branches_and_ancillas_are_proved() {
    let t = support::verified(
        "fn spread(a: &mut [i64; 8], i: &mut i64, k: &i64) {
            from i == 0 {
                ancilla t: i64 = 0 {
                    t += k;
                    a[i] += t;
                    t -= k;
                }
                if k > 0 { a[i] += 1; } else { a[i] -= 1; } fi k > 0;
            } loop { i += 1; } until i == 7;
         }",
    );
    assert_eq!(t.reversible, ["spread"]);
}

#[test]
fn a_function_calling_a_function_with_a_loop_uses_its_lemmas() {
    let t = support::verified(
        "fn count(x: &mut i64, i: &mut i64, n: &i64) {
            from i == 0 { x += 2; } loop { i += 1; } until i == n;
         }
         fn twice(x: &mut i64, i: &mut i64, n: &i64) {
            call count(x, i, n);
            i -= n;
            call count(x, i, n);
         }",
    );
    assert_eq!(t.reversible, ["count", "twice"]);
}

#[test]
fn lean_rejects_a_loop_whose_body_is_not_reversible() {
    let t = support::translation(
        "fn f(x: &mut i64, y: &mut i64, i: &mut i64) {
            from i == 0 {
                if x > 0 { y += x; } else { y -= x; } fi y > 0;
            } loop { i += 1; } until i == 3;
         }",
    );
    assert!(t.skipped.is_empty());
    assert!(
        support::lean_accepts(&t).is_err(),
        "Lean should not prove this loop reversible"
    );
}

#[test]
fn lean_rejects_a_loop_with_a_corrupted_inverse_body() {
    let mut t = support::translation(
        "fn count(x: &mut i64, i: &mut i64, n: &i64) {
            from i == 0 { x += 2; } loop { i += 1; } until i == n;
         }",
    );
    assert!(support::lean_accepts(&t).is_ok());
    t.lean = t.lean.replace(
        "\u{ab}x\u{bb} := (\u{ab}x\u{bb} - (2 : Roop.I64))",
        "\u{ab}x\u{bb} := (\u{ab}x\u{bb} - (3 : Roop.I64))",
    );
    assert!(
        support::lean_accepts(&t).is_err(),
        "the loop lemma must depend on the body being inverted correctly"
    );
}

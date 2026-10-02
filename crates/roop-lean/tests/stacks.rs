mod support;

#[test]
fn push_pop_and_logged_updates_are_proved_reversible() {
    let t = support::verified(
        "fn park(x: &mut i64, s: &mut Stack<i64, 4>) { push s <- x; }
         fn fetch(x: &mut i64, s: &mut Stack<i64, 4>) { pop s -> x; }
         fn crush(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 4>) {
             logged h { x = y + 1; y %= 5; }
         }",
    );
    assert_eq!(t.reversible, ["park", "fetch", "crush"]);
    assert!(t.forward_only.is_empty());
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn a_function_that_destroys_information_is_undone_by_its_history() {
    let t = support::verified(
        "fn scale(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 4>) {
             logged h { x = y * 2; }
         }
         fn keep(x: &mut i64, y: &mut i64, r: &mut i64) {
             ancilla h: Stack<i64, 4> = empty {
                 call scale(x, y, h);
                 r += x;
                 uncall scale(x, y, h);
             }
         }",
    );
    assert_eq!(t.reversible, ["scale", "keep"]);
}

#[test]
fn lean_rejects_an_inverse_that_restores_the_wrong_value() {
    let mut t = support::translation(
        "fn crush(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 4>) {
             logged h { x = y + 1; }
         }",
    );
    assert!(support::lean_accepts(&t).is_ok());
    t.lean = t.lean.replace(
        "\u{ab}x\u{bb} := __pop1.1",
        "\u{ab}x\u{bb} := (__pop1.1 + (1 : Roop.I64))",
    );
    assert!(
        support::lean_accepts(&t).is_err(),
        "the history must give back exactly what was destroyed"
    );
}

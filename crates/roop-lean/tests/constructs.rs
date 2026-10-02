mod support;

#[test]
fn branches_with_exit_assertions_are_proved_reversible() {
    let t = support::verified(
        "fn f(x: &mut i64, y: &mut i64) {
            if x > 0 { y += x; } else { y -= x; } fi x > 0;
         }",
    );
    assert_eq!(t.reversible, ["f"]);
}

#[test]
fn matches_on_enums_and_ints_are_proved_reversible() {
    support::verified(
        "enum S { A, B }
         fn f(s: &S, y: &mut i64) {
             match s {
                 S::A => { y += 1; } assert y == 11;
                 S::B => { y += 2; } assert y == 12;
             }
         }",
    );
}

#[test]
fn ancillas_are_checked_and_never_fail() {
    support::verified(
        "fn f(x: &mut i64, y: &mut i64) {
            ancilla t: i64 = 0 { t += x; y += t; t -= x; }
         }",
    );
}

#[test]
fn borrows_write_back_to_arrays_and_fields() {
    support::verified(
        "struct P { a: i64, b: i64 }
         fn f(p: &mut P, a: &mut [i64; 4], k: &i64) {
            borrow c = p.a { c += k; }
            borrow d = a[2] { d += k; }
            p.b += k;
            a[1] += p.a;
         }",
    );
}

#[test]
fn calls_and_uncalls_use_the_callee_and_its_inverse() {
    support::verified(
        "fn inc(x: &mut i64, k: &i64) { x += k; }
         fn twice(x: &mut i64, y: &mut i64, k: &i64) {
            call inc(x, k);
            call inc(y, k);
            uncall inc(x, k);
         }",
    );
}

#[test]
fn callers_may_come_before_their_callees() {
    support::verified(
        "fn outer(x: &mut i64, k: &i64) { call inner(x, k); }
         fn inner(x: &mut i64, k: &i64) { x += k; }",
    );
}

#[test]
fn array_updates_with_computed_indices_are_proved_reversible() {
    support::verified("fn f(a: &mut [i64; 8], i: &i64, k: &i64) { a[i] += k; a[i] ^= k; }");
}

#[test]
fn booleans_and_floats_translate() {
    let t = support::translation("fn f(b: &mut bool, x: &mut f64, k: &f64) { b ^= true; x += k; }");
    assert!(t.skipped.is_empty(), "{:?}", t.skipped);
    assert!(support::lean_accepts(&t).is_ok());
}

mod support;

#[test]
fn a_parallel_loop_is_proved_order_independent() {
    let t = support::verified(
        "fn axpy(a: &mut [i64; 8], b: &[i64; 8], k: &i64, i: &mut i64) {
            #[parallel] from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == 7;
         }",
    );
    assert_eq!(t.parallel, ["axpy"]);
    assert!(t.lean.contains("_commute"));
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn disjoint_writes_through_ancillas_shifts_fields_and_branches_commute() {
    let t = support::verified(
        "struct Cloud { xs: [i64; 8], ys: [i64; 8] }
         fn mix(a: &mut [i64; 8], b: &mut [i64; 8], c: &[i64; 8], k: &i64, i: &mut i64) {
            #[parallel] from i == 0 {
                ancilla t: i64 = 0 { t += c[i] * k; a[i] += t; b[i] -= t; t -= c[i] * k; }
            } loop { i += 1; } until i == 7;
         }
         fn shift(a: &mut [i64; 9], b: &[i64; 9], i: &mut i64) {
            #[parallel] from i == 0 { a[i + 1] += b[i]; } loop { i += 1; } until i == 7;
         }
         fn swapcols(p: &mut Cloud, i: &mut i64) {
            #[parallel] from i == 0 { p.xs[i] <=> p.ys[i]; } loop { i += 1; } until i == 7;
         }
         fn branchy(a: &mut [i64; 8], k: &i64, i: &mut i64) {
            #[parallel] from i == 0 {
                if k > 0 { a[i] += 1; } else { a[i] -= 1; } fi k > 0;
            } loop { i += 1; } until i == 7;
         }",
    );
    assert_eq!(t.parallel, ["mix", "shift", "swapcols", "branchy"]);
}

#[test]
fn a_sequential_loop_gets_no_commutation_theorem() {
    let t = support::verified(
        "fn acc(a: &mut [i64; 8], i: &mut i64) {
            from i == 1 { a[i] += a[i - 1]; } loop { i += 1; } until i == 7;
         }",
    );
    assert!(t.parallel.is_empty());
    assert!(!t.lean.contains("_commute"));
}

#[test]
fn lean_rejects_a_parallel_loop_with_a_loop_carried_dependence() {
    let src = "fn f(a: &mut [i64; 8], i: &mut i64) {
        #[parallel] from i == 0 { a[i] += a[0]; } loop { i += 1; } until i == 7;
     }";
    assert!(
        roop_check::check(&roop_syntax::parse(src).unwrap()).is_err(),
        "the checker already refuses this"
    );
    let t = support::unchecked(src);
    assert!(
        support::lean_accepts(&t).is_err(),
        "iteration 0 changes what the others read, so they do not commute"
    );
}

#[test]
fn lean_rejects_a_parallel_loop_that_accumulates_into_a_shared_variable() {
    let src = "fn f(a: &[i64; 8], s: &mut i64, i: &mut i64) {
        #[parallel] from i == 0 { s += a[i]; s ^= 1; } loop { i += 1; } until i == 7;
     }";
    let t = support::unchecked(src);
    assert!(support::lean_accepts(&t).is_err());
}

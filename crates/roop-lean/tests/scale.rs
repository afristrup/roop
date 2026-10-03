mod support;

#[test]
fn guarded_scaling_is_proved_reversible_in_both_directions() {
    let t = support::verified(
        "fn scale(x: &mut i64, k: &i64) { x *= k; }
         fn shrink(x: &mut i64, k: &i64) { x /= k; }
         fn both(x: &mut i64, y: &mut i64, k: &i64) { x *= k; y += x; y -= x; x /= k; }",
    );
    assert_eq!(t.reversible, ["scale", "shrink", "both"]);
    assert!(!t.lean.contains("sorry"));
}

#[test]
fn scaling_inside_loops_and_arrays_is_proved() {
    support::verified(
        "fn scale_all(a: &mut [i64; 4], k: &i64, i: &mut i64) {
             from i == 0 { a[i] *= k; } loop { i += 1; } until i == 3;
         }",
    );
}

#[test]
fn float_scaling_is_translated_but_claims_no_roundtrip() {
    let t = support::translation("fn grow(x: &mut f64, k: &f64) { x *= k; }");
    assert_eq!(t.inexact, ["grow"]);
    assert!(support::lean_accepts(&t).is_ok());
}

#[test]
fn a_struct_with_a_float_inside_makes_its_functions_inexact() {
    let t = support::translation(
        "struct Point { x: f64, y: f64 }
         struct Segment { start: Point, stop: Point }
         struct Counter { n: i64 }
         fn nudge(s: &mut Segment, d: &f64) { s.start.x += d; }
         fn bump(c: &mut Counter, k: &i64) { c.n += k; }",
    );
    assert_eq!(t.inexact, ["nudge"]);
    assert_eq!(t.reversible, ["bump"]);
    assert!(support::lean_accepts(&t).is_ok());
}

mod support;

#[test]
fn updates_swaps_and_xor_are_proved_reversible() {
    let t = support::verified(
        "fn add(x: &mut i64, k: &i64) { x += k; }
         fn mix(x: &mut i64, y: &mut i64, k: &i64) {
             y += x;
             x <=> y;
             x ^= k;
         }",
    );
    assert_eq!(t.reversible, ["add", "mix"]);
}

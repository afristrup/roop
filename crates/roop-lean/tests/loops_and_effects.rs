mod support;

#[test]
fn irreversible_functions_translate_forward_only() {
    let t = support::verified(
        "irrev fn crush(x: &mut i64, y: &mut i64) { x = y + 1; y *= 3; y %= 5; }
         fn mix(a: &mut i64, b: &mut i64) { a += 5; irrev { b = 0; } }",
    );
    assert_eq!(t.forward_only, ["crush", "mix"]);
    assert!(t.reversible.is_empty());
    assert!(!t.lean.contains("crush_inv"));
}

#[test]
fn constructs_that_are_not_modelled_are_skipped_not_fatal() {
    let t = support::translation(
        "fn ok(x: &mut i64, k: &i64) { x += k; }
         irrev fn guarded(x: &mut i64, y: &mut i64) {
            try { if x > 0 { y += 1; } fi y > 0; } catch_rollback { y += 2; }
         }
         fn pipe(x: &mut i64, y: &mut i64) {
            chan c: i64 {
                #[concurrent] { send c <- x; }
                #[concurrent] { recv c -> y; }
            }
         }
         fn uses_skipped(x: &mut i64, y: &mut i64) { irrev { call guarded(x, y); } }",
    );
    assert_eq!(t.reversible, ["ok"]);
    let names: Vec<&str> = t.skipped.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["guarded", "pipe", "uses_skipped"]);
    assert!(t.skipped[0].1.contains("try"));
    assert!(t.skipped[1].1.contains("channels") || t.skipped[1].1.contains("concurrent"));
    assert!(support::lean_accepts(&t).is_ok());
}

#[test]
fn the_repository_examples_translate_and_check() {
    let root = format!("{}/../../roop", env!("CARGO_MANIFEST_DIR"));
    for file in [
        "std/mod.roop",
        "examples/insurance/mod.roop",
        "examples/history/mod.roop",
    ] {
        let src = std::fs::read_to_string(format!("{root}/{file}")).unwrap();
        let t = support::translation(&src);
        assert!(support::lean_accepts(&t).is_ok(), "{file}");
        assert!(!t.reversible.is_empty(), "{file}");
    }
}

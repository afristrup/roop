use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(src).unwrap())
}

const SIGNAL: &str = "enum Signal { High, Low, Off }";

fn with_match(arms: &str) -> String {
    format!("{SIGNAL} fn f(s: &Signal, y: &mut i64) {{ match s {{ {arms} }} }}")
}

#[test]
fn accepts_match_covering_every_variant() {
    let arms = "Signal::High => { y += 1; } assert y == 1;
                Signal::Low => { y -= 1; } assert y == 0;
                Signal::Off => { } assert y == 2;";
    assert_eq!(run(&with_match(arms)), Ok(()));
}

#[test]
fn rejects_match_missing_a_variant() {
    let arms = "Signal::High => { } assert y == 1; Signal::Low => { } assert y == 0;";
    assert!(matches!(
        run(&with_match(arms)),
        Err(CheckError::NonExhaustiveMatch { .. })
    ));
}

#[test]
fn wildcard_covers_remaining_variants() {
    let arms = "Signal::High => { } assert y == 1; _ => { } assert y == 0;";
    assert_eq!(run(&with_match(arms)), Ok(()));
}

#[test]
fn both_bool_arms_are_exhaustive() {
    let src = "fn f(b: &bool, y: &mut i64) {
        match b { true => { y += 1; } assert y == 1; false => { y -= 1; } assert y == 0; }
    }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn rejects_unknown_variant_in_pattern() {
    let arms = "Signal::Blue => { } assert y == 1; _ => { } assert y == 0;";
    assert!(matches!(
        run(&with_match(arms)),
        Err(CheckError::UnknownVariant { variant, .. }) if variant == "Blue"
    ));
}

#[test]
fn rejects_unknown_enum_in_assertion() {
    let src = "fn f(y: &mut i64) { if y > 0 { y += 1; } fi y == Mode::On; }";
    assert!(matches!(run(src), Err(CheckError::UnknownVariant { .. })));
}

#[test]
fn rejects_duplicate_variants() {
    assert!(matches!(
        run("enum E { A, B, A }"),
        Err(CheckError::DuplicateVariant { variant, .. }) if variant == "A"
    ));
}

#[test]
fn checks_variants_inside_nested_blocks() {
    let src = format!(
        "{SIGNAL} fn f(y: &mut i64) {{ try {{ if y > 0 {{ y += 1; }} fi y == Signal::Nope; }} catch_rollback {{ }} }}"
    );
    assert!(matches!(run(&src), Err(CheckError::UnknownVariant { .. })));
}

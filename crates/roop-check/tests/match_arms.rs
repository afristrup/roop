use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("fn f(x: &mut i64, y: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_match_with_wildcard() {
    assert_eq!(
        run("match x { 0 => { y += 1; } assert y == 1; _ => { y -= 1; } assert y != 1; }"),
        Ok(())
    );
}

#[test]
fn rejects_match_without_wildcard() {
    assert!(matches!(
        run("match x { 0 => { y += 1; } assert y == 1; }"),
        Err(CheckError::NonExhaustiveMatch { .. })
    ));
}

#[test]
fn checks_arm_bodies() {
    assert!(run("match x { _ => { y += y; } assert y == 0; }").is_err());
}

#[test]
fn ancilla_modified_in_match_arm_is_rejected() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { match x { _ => { t += 1; } assert t == 1; } }"),
        Err(CheckError::AncillaTouchedInControlFlow { .. })
    ));
}

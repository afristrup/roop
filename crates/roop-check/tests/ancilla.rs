use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("rev fn f(x: &mut i64, y: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_compute_use_uncompute() {
    assert_eq!(
        run("ancilla t: i64 = 0 { t += x; y += t; t -= x; }"),
        Ok(())
    );
}

#[test]
fn accepts_nested_inverse_order() {
    assert_eq!(
        run("ancilla t: i64 = 0 { t += x; t ^= y; t ^= y; t -= x; }"),
        Ok(())
    );
}

#[test]
fn rejects_unrestored_ancilla() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { t += x; y += t; }"),
        Err(CheckError::AncillaNotRestored { .. })
    ));
}

#[test]
fn rejects_wrong_inverse() {
    assert!(run("ancilla t: i64 = 0 { t += x; t -= y; }").is_err());
}

#[test]
fn rejects_uncompute_after_input_changed() {
    assert!(run("ancilla t: i64 = 0 { t += x; x += 1; t -= x; }").is_err());
}

#[test]
fn rejects_ancilla_modified_in_control_flow() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { if x > 0 { t += 1; } fi x > 0; }"),
        Err(CheckError::AncillaTouchedInControlFlow { .. })
    ));
}

#[test]
fn accepts_call_then_uncall() {
    assert_eq!(
        run("ancilla t: i64 = 0 { call g(t, x); y += t; uncall g(t, x); }"),
        Ok(())
    );
}

use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("fn f(x: &mut i64, y: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_try_with_failable_body() {
    assert_eq!(
        run("try { if x > 0 { y += x; } fi y > 0; } catch_rollback { y += 1; }"),
        Ok(())
    );
}

#[test]
fn accepts_try_around_a_call() {
    assert_eq!(run("try { call g(x); } catch_rollback { y += 1; }"), Ok(()));
}

#[test]
fn rejects_try_that_cannot_fail() {
    assert!(matches!(
        run("try { y += x; } catch_rollback { y -= x; }"),
        Err(CheckError::TryCannotFail { .. })
    ));
}

#[test]
fn nested_try_absorbs_inner_failures() {
    let inner = "try { call g(x); } catch_rollback { y += 1; }";
    assert!(run(&format!("try {{ {inner} }} catch_rollback {{ y += 2; }}")).is_err());
}

#[test]
fn checks_body_and_handler() {
    assert!(run("try { call g(x); y += y; } catch_rollback { }").is_err());
    assert!(run("try { call g(x); } catch_rollback { y += y; }").is_err());
}

#[test]
fn ancilla_modified_in_try_is_rejected() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { try { call g(t); } catch_rollback { } }"),
        Err(CheckError::AncillaTouchedInControlFlow { .. })
    ));
}

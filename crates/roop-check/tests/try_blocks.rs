use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("fn f(x: &mut i64, y: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_try_with_failable_body() {
    assert_eq!(
        run("irrev { try { if x > 0 { y += 1; } fi y > 0; } catch_rollback { y += 1; } }"),
        Ok(())
    );
}

#[test]
fn accepts_try_around_a_call() {
    assert_eq!(
        run("irrev { try { call g(x); } catch_rollback { y += 1; } }"),
        Ok(())
    );
}

#[test]
fn rejects_try_that_cannot_fail() {
    assert!(matches!(
        run("irrev { try { y += x; } catch_rollback { y -= x; } }"),
        Err(CheckError::TryCannotFail { .. })
    ));
}

#[test]
fn nested_try_absorbs_inner_failures() {
    let inner = "try { call g(x); } catch_rollback { y += 1; }";
    assert!(
        run(&format!(
            "irrev {{ try {{ {inner} }} catch_rollback {{ y += 2; }} }}"
        ))
        .is_err()
    );
}

#[test]
fn irrev_lifts_the_reversibility_rules_inside_a_try() {
    // `y += y` destroys information, which irreversible code may do.
    assert_eq!(
        run("irrev { try { call g(x); y += y; } catch_rollback { y += y; } }"),
        Ok(())
    );
}

#[test]
fn a_try_in_reversible_code_is_rejected() {
    assert!(matches!(
        run("try { call g(x); } catch_rollback { y += 1; }"),
        Err(CheckError::IrreversibleOutsideIrrev { .. })
    ));
}

#[test]
fn an_irrev_block_may_not_touch_an_enclosing_ancilla() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { irrev { try { call g(t); } catch_rollback { } } }"),
        Err(CheckError::AncillaTouchedInControlFlow { .. })
    ));
}

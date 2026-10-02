use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(src).unwrap())
}

const HEAD: &str = "fn f(x: &mut i64, y: &mut i64, h: &mut Stack<i64, 8>, failed: &mut bool)";

#[test]
fn logged_blocks_make_destroying_updates_reversible() {
    assert_eq!(
        run(&format!("{HEAD} {{ logged h {{ x = y + 1; y %= 5; }} }}")),
        Ok(())
    );
}

#[test]
fn destroying_updates_outside_logged_still_need_irrev() {
    assert!(matches!(
        run(&format!("{HEAD} {{ x = y + 1; }}")),
        Err(CheckError::IrreversibleOutsideIrrev { .. })
    ));
}

#[test]
fn a_logged_update_may_not_read_what_it_destroys() {
    assert!(matches!(
        run(&format!("{HEAD} {{ logged h {{ x = x + 1; }} }}")),
        Err(CheckError::SelfReferentialUpdate { .. })
    ));
}

#[test]
fn a_function_with_logged_blocks_still_has_an_inverse() {
    let program = parse(&format!("{HEAD} {{ logged h {{ x = y + 1; }} }}")).unwrap();
    assert!(roop_check::irreversible_fns(&program).is_empty());
}

#[test]
fn an_ancilla_stack_must_be_emptied() {
    assert_eq!(
        run(&format!(
            "{HEAD} {{ ancilla g: Stack<i64, 4> = empty {{ push g <- x; pop g -> x; }} }}"
        )),
        Ok(())
    );
    assert!(matches!(
        run(&format!(
            "{HEAD} {{ ancilla g: Stack<i64, 4> = empty {{ push g <- x; }} }}"
        )),
        Err(CheckError::AncillaNotRestored { .. })
    ));
}

#[test]
fn a_try_with_an_outcome_is_reversible_without_irrev() {
    assert_eq!(
        run(&format!(
            "{HEAD} {{ try {{ if x > 0 {{ y += 1; }} fi y > 0; }} catch_rollback {{ y += 2; }} -> failed; }}"
        )),
        Ok(())
    );
}

#[test]
fn a_try_without_an_outcome_still_needs_irrev() {
    assert!(matches!(
        run(&format!(
            "{HEAD} {{ try {{ if x > 0 {{ y += 1; }} fi y > 0; }} catch_rollback {{ y += 2; }} }}"
        )),
        Err(CheckError::IrreversibleOutsideIrrev { .. })
    ));
}

#[test]
fn the_outcome_belongs_to_the_try() {
    assert!(matches!(
        run(&format!(
            "{HEAD} {{ try {{ if x > 0 {{ y += 1; }} fi y > 0; failed ^= true; }} catch_rollback {{ y += 2; }} -> failed; }}"
        )),
        Err(CheckError::TryOutcomeWritten { .. })
    ));
}

#[test]
fn a_try_with_an_outcome_cannot_hold_what_it_could_not_undo() {
    for body in [
        "irrev { y = 1; } if x > 0 { y += 1; } fi y > 0;",
        "y = 1; if x > 0 { y += 1; } fi y > 0;",
        "try { if x > 0 { y += 1; } fi y > 0; } catch_rollback { y += 1; }",
    ] {
        let src = format!("{HEAD} {{ try {{ {body} }} catch_rollback {{ y += 2; }} -> failed; }}");
        assert!(
            matches!(run(&src), Err(CheckError::NotUnwindable { .. })),
            "{body}"
        );
    }
}

#[test]
fn a_call_to_something_that_cannot_be_undone_is_not_allowed_in_such_a_try() {
    let src = "irrev fn crush(x: &mut i64) { x = 0; }
               fn f(x: &mut i64, y: &mut i64, failed: &mut bool) {
                   try { irrev { call crush(x); } } catch_rollback { y += 1; } -> failed;
               }";
    assert!(matches!(run(src), Err(CheckError::NotUnwindable { .. })));
}

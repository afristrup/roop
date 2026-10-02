use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(src).unwrap())
}

#[test]
fn a_plain_function_is_reversible_and_rejects_overwrites() {
    for stmt in ["x = 1;", "x %= 2;"] {
        let src = format!("fn f(x: &mut i64) {{ {stmt} }}");
        assert!(
            matches!(run(&src), Err(CheckError::IrreversibleOutsideIrrev { .. })),
            "{stmt}"
        );
    }
}

#[test]
fn irrev_functions_may_overwrite() {
    assert_eq!(run("irrev fn f(x: &mut i64) { x = 1; x *= 3; }"), Ok(()));
}

#[test]
fn irrev_blocks_may_overwrite_inside_a_plain_function() {
    assert_eq!(
        run("fn f(x: &mut i64, y: &mut i64) { y += 1; irrev { x = 0; } }"),
        Ok(())
    );
}

#[test]
fn irrev_lifts_non_interference_and_ancilla_restoration() {
    let body = "x += x; ancilla t: i64 = 0 { t += 1; }";
    assert_eq!(
        run(&format!("irrev fn f(x: &mut i64) {{ {body} }}")),
        Ok(())
    );
    assert!(run(&format!("fn f(x: &mut i64) {{ {body} }}")).is_err());
}

#[test]
fn irrev_does_not_lift_the_safety_rules() {
    let racy = "irrev fn f(a: &mut [i64; 8], s: &mut i64, i: &mut i64) {
        #[parallel] from i == 0 { s += a[i]; } loop { i += 1; } until i == 7;
    }";
    assert!(matches!(
        run(racy),
        Err(CheckError::ParallelWriteNotDisjoint { .. })
    ));
    let shared = "irrev fn f(x: &mut i64) { #[concurrent] { x = 1; } #[concurrent] { x = 2; } }";
    assert!(matches!(
        run(shared),
        Err(CheckError::ConcurrentConflict { .. })
    ));
}

#[test]
fn reversible_code_cannot_call_an_irreversible_function() {
    let src = "irrev fn wipe(x: &mut i64) { x = 0; }
               fn f(x: &mut i64) { call wipe(x); }";
    assert!(
        matches!(run(src), Err(CheckError::CallsIrreversible { callee, .. }) if callee == "wipe")
    );
}

#[test]
fn irrev_code_may_call_an_irreversible_function() {
    let src = "irrev fn wipe(x: &mut i64) { x = 0; }
               fn f(x: &mut i64) { irrev { call wipe(x); } }
               irrev fn g(x: &mut i64) { call wipe(x); }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn a_function_with_an_irrev_block_is_itself_irreversible() {
    let src = "fn leaky(x: &mut i64) { irrev { x = 0; } }
               fn f(x: &mut i64) { call leaky(x); }";
    assert!(
        matches!(run(src), Err(CheckError::CallsIrreversible { callee, .. }) if callee == "leaky")
    );
}

#[test]
fn nothing_can_uncall_an_irreversible_function() {
    let src = "irrev fn wipe(x: &mut i64) { x = 0; }
               irrev fn f(x: &mut i64) { uncall wipe(x); }";
    assert!(matches!(
        run(src),
        Err(CheckError::UncallIrreversible { .. })
    ));
}

#[test]
fn constructors_must_stay_reversible() {
    let src = "struct P { a: i64,
        build(x: &i64) { irrev { self.a = 1; } }
        unbuild(x: &i64) { irrev { self.a = 0; } } }";
    assert!(matches!(
        run(src),
        Err(CheckError::IrreversibleOutsideIrrev { .. })
    ));
}

#[test]
fn reversible_calls_between_plain_functions_are_fine() {
    let src = "fn inc(x: &mut i64) { x += 1; }
               fn twice(x: &mut i64) { call inc(x); call inc(x); uncall inc(x); }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn scaling_is_reversible_and_needs_no_irrev() {
    assert_eq!(
        run("fn f(x: &mut i64, k: &i64) { x *= k; x /= 3; }"),
        Ok(())
    );
}

#[test]
fn a_scaled_place_may_not_appear_in_its_own_factor() {
    assert!(run("fn f(x: &mut i64) { x *= x; }").is_err());
}

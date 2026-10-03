use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(src).unwrap())
}

#[test]
fn keeping_an_ancilla_releases_it_without_an_inverse() {
    assert_eq!(
        run("fn f(a: &i64) { ancilla x: i64 = 0; x += a; keep x; }"),
        Ok(())
    );
}

#[test]
fn a_kept_ancilla_may_be_changed_in_control_flow_before_it() {
    let src = "fn f(a: &i64) {
        ancilla x: i64 = 0;
        if a > 0 { x += a; } fi a > 0;
        keep x;
    }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn a_loop_that_ends_each_round_by_keeping_releases_it() {
    let src = "fn f(i: &mut i64) {
        ancilla x: i64 = 0;
        from i == 0 { x += 1; keep x; } loop { i += 1; } until i == 3;
    }";
    assert_eq!(run(src), Ok(()));
}

#[test]
fn a_change_after_the_keep_still_has_to_be_undone() {
    assert!(matches!(
        run("fn f(a: &i64) { ancilla x: i64 = 0; x += a; keep x; x += a; }"),
        Err(CheckError::AncillaNotRestored { .. })
    ));
}

#[test]
fn keeping_does_not_release_an_ancilla_that_starts_nonzero() {
    assert!(matches!(
        run("fn f(a: &i64) { ancilla x: i64 = 1; x += a; keep x; }"),
        Err(CheckError::AncillaNotRestored { .. })
    ));
}

#[test]
fn keeping_part_of_an_ancilla_does_not_release_it() {
    assert!(matches!(
        run("fn f(a: &i64) { ancilla x: [i64; 2] = 0; x[0] += a; keep x[0]; }"),
        Err(CheckError::AncillaNotRestored { .. })
    ));
}

#[test]
fn a_parallel_loop_may_not_keep() {
    let src = "fn f(i: &mut i64, a: &mut i64) {
        #[parallel] from i == 0 { keep a; } loop { i += 1; } until i == 3;
    }";
    assert!(matches!(run(src), Err(CheckError::WorldInParallel { .. })));
}

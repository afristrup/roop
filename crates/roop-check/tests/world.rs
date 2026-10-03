use roop_check::{CheckError, check};
use roop_syntax::parse;

const PRELUDE: &str = "
extern world fn roop_out(fd: &i64, buf: &[u8; 2], len: &i64);
extern fn roop_commit();
fn say(s: &[u8; 2]) { call roop_out(1, s, 2); }
";

fn run(src: &str) -> Result<(), CheckError> {
    check(&parse(&format!("{PRELUDE}{src}")).unwrap())
}

#[test]
fn reversible_code_may_call_a_world_function_and_what_calls_one() {
    assert_eq!(
        run("fn f(s: &[u8; 2]) { call say(s); call roop_out(2, s, 2); }"),
        Ok(())
    );
}

#[test]
fn reversible_code_may_not_call_an_irreversible_extern() {
    assert!(matches!(
        run("fn f() { call roop_commit(); }"),
        Err(CheckError::CallsIrreversible { .. })
    ));
}

#[test]
fn irrev_code_may_commit() {
    assert_eq!(run("irrev fn f() { call roop_commit(); }"), Ok(()));
}

#[test]
fn a_parallel_loop_may_not_change_the_world() {
    let src = "fn f(s: &[u8; 2], i: &mut i64) {
        #[parallel] from i == 0 { call roop_out(1, s, 2); } loop { i += 1; } until i == 3;
    }";
    assert!(
        matches!(run(src), Err(CheckError::WorldInParallel { .. })),
        "{:?}",
        run(src)
    );
}

#[test]
fn nor_may_it_reach_the_world_through_another_function() {
    let src = "fn f(s: &[u8; 2], i: &mut i64) {
        #[parallel] from i == 0 { call say(s); } loop { i += 1; } until i == 3;
    }";
    assert!(matches!(run(src), Err(CheckError::WorldInParallel { .. })));
}

#[test]
fn a_concurrent_task_may_not_change_the_world() {
    let src = "fn f(s: &[u8; 2]) {
        chan c: i64 {
            #[concurrent] { call say(s); }
            #[concurrent] { }
        }
    }";
    assert!(
        matches!(run(src), Err(CheckError::WorldInParallel { .. })),
        "{:?}",
        run(src)
    );
}

#[test]
fn a_try_may_wrap_a_world_function() {
    let src = "fn f(s: &[u8; 2], failed: &mut bool) {
        try { call say(s); expect false; } catch_rollback { } -> failed;
    }";
    assert_eq!(run(src), Ok(()));
}

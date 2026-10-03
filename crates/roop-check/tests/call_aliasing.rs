use roop_check::{CheckError, check};
use roop_syntax::parse;

const G: &str = "fn g(x: &mut i64, y: &i64) { x += y; }
fn swap2(a: &mut i64, b: &mut i64) { a <=> b; }";

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!(
        "{G} fn f(a: &mut i64, b: &mut i64, v: &mut [i64; 4], i: &i64, j: &i64) {{ {body} }}"
    );
    check(&parse(&src).unwrap())
}

#[test]
fn different_variables_are_fine() {
    assert_eq!(run("call g(a, b);"), Ok(()));
    assert_eq!(run("call g(v[0], v[1]);"), Ok(()));
    assert_eq!(run("call g(a, i);"), Ok(()));
}

#[test]
fn a_variable_given_twice_is_refused() {
    assert!(matches!(
        run("call g(a, a);"),
        Err(CheckError::CallAliasing { .. })
    ));
    assert!(matches!(
        run("call swap2(a, a);"),
        Err(CheckError::CallAliasing { .. })
    ));
}

#[test]
fn a_variable_read_by_another_argument_is_refused() {
    assert!(matches!(
        run("call g(a, a + 1);"),
        Err(CheckError::CallAliasing { .. })
    ));
}

#[test]
fn elements_with_indices_that_may_be_equal_are_refused() {
    assert!(matches!(
        run("call g(v[i], v[j]);"),
        Err(CheckError::CallAliasing { .. })
    ));
    assert!(matches!(
        run("call g(v[i], v[i]);"),
        Err(CheckError::CallAliasing { .. })
    ));
}

#[test]
fn an_index_changed_by_the_same_call_is_refused() {
    assert!(matches!(
        run("call g(i, v[i]);"),
        Err(CheckError::CallAliasing { .. })
    ));
}

#[test]
fn uncall_is_checked_too() {
    assert!(matches!(
        run("uncall g(a, a);"),
        Err(CheckError::CallAliasing { .. })
    ));
}

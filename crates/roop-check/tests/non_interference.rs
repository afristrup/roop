use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("rev fn f(x: &mut i64, y: &mut i64, a: &mut [i64; 4]) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn allows_updates_from_other_variables() {
    assert_eq!(run("x += y + 2;"), Ok(()));
    assert_eq!(run("a[0] += x;"), Ok(()));
}

#[test]
fn rejects_self_referential_update() {
    assert!(matches!(
        run("x += x * 2;"),
        Err(CheckError::SelfReferentialUpdate { var, .. }) if var == "x"
    ));
}

#[test]
fn rejects_array_element_depending_on_itself() {
    assert!(run("a[0] += a[1];").is_err());
    assert!(run("a[a[0]] += 1;").is_err());
}

#[test]
fn checks_nested_blocks() {
    assert!(run("if y > 0 { x += x; } fi y > 0;").is_err());
}

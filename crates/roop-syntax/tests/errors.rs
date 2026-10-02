use roop_syntax::parse;

#[test]
fn rejects_destructive_assignment() {
    assert!(parse("rev fn f(x: &mut i64) { x = 1; }").is_err());
}

#[test]
fn rejects_if_without_exit_assertion() {
    assert!(parse("rev fn f(x: &mut i64) { if x > 0 { x += 1; } }").is_err());
}

#[test]
fn rejects_from_without_until() {
    assert!(parse("rev fn f(x: &mut i64) { from x == 0 { x += 1; } }").is_err());
}

#[test]
fn rejects_unknown_character() {
    let err = parse("rev fn f() { # }").unwrap_err();
    assert_eq!(err.span.start, 13);
}

#[test]
fn rejects_match_arm_without_exit_assertion() {
    assert!(parse("rev fn f(x: &mut i64) { match x { _ => { x += 1; } } }").is_err());
}

#[test]
fn rejects_empty_match() {
    assert!(parse("rev fn f(x: &mut i64) { match x { } }").is_err());
}

#[test]
fn rejects_try_without_handler() {
    assert!(parse("rev fn f(x: &mut i64) { try { x += 1; } }").is_err());
}

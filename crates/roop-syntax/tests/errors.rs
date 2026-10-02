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

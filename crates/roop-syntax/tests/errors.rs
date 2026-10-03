use roop_syntax::parse;

#[test]
fn rejects_if_without_exit_assertion() {
    assert!(parse("fn f(x: &mut i64) { if x > 0 { x += 1; } }").is_err());
}

#[test]
fn rejects_from_without_until() {
    assert!(parse("fn f(x: &mut i64) { from x == 0 { x += 1; } }").is_err());
}

#[test]
fn rejects_unknown_character() {
    let err = parse("fn f() { @ }").unwrap_err();
    assert_eq!(err.span.start, 9);
}

#[test]
fn rejects_match_arm_without_exit_assertion() {
    assert!(parse("fn f(x: &mut i64) { match x { _ => { x += 1; } } }").is_err());
}

#[test]
fn rejects_empty_match() {
    assert!(parse("fn f(x: &mut i64) { match x { } }").is_err());
}

#[test]
fn rejects_try_without_handler() {
    assert!(parse("fn f(x: &mut i64) { try { x += 1; } }").is_err());
}

#[test]
fn rejects_borrow_without_source() {
    assert!(parse("fn f(x: &mut i64) { borrow c { c += 1; } }").is_err());
}

#[test]
fn rejects_unknown_attribute_and_target() {
    assert!(parse("fn f(x: &mut i64) { #[inline] x += 1; }").is_err());
    assert!(parse("fn f(x: &mut i64) { #[parallel(tpu)] x += 1; }").is_err());
}

#[test]
fn rejects_malformed_channel_statements() {
    assert!(parse("fn f(x: &mut i64) { send c x; }").is_err());
    assert!(parse("fn f(x: &mut i64) { recv c <- x; }").is_err());
    assert!(parse("fn f(x: &mut i64) { chan c { } }").is_err());
}

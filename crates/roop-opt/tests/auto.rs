use roop_check::check;
use roop_opt::{AutoError, expand_auto};
use roop_syntax::parse;

fn expanded(src: &str) -> Result<roop_syntax::Program, AutoError> {
    expand_auto(&parse(src).unwrap())
}

#[test]
fn an_auto_ancilla_is_let_go_of_at_the_end_of_its_block() {
    let program = expanded("fn f(a: &i64) { auto ancilla x: i64 = 0; x += a; }").unwrap();
    assert_eq!(check(&program), Ok(()));
}

#[test]
fn one_declared_in_a_loop_body_is_let_go_of_each_round() {
    let src = "fn f(i: &mut i64, a: &i64) {
        from i == 0 {
            auto ancilla x: i64 = 0;
            x += a;
        } loop { i += 1; } until i == 3;
        i -= 3;
    }";
    assert_eq!(check(&expanded(src).unwrap()), Ok(()));
}

#[test]
fn without_auto_the_same_ancilla_is_rejected() {
    let src = "fn f(a: &i64) { ancilla x: i64 = 0; x += a; }";
    assert!(check(&parse(src).unwrap()).is_err());
}

#[test]
fn an_auto_ancilla_must_start_at_zero() {
    assert_eq!(
        expanded("fn f() { auto ancilla x: i64 = 1; }"),
        Err(AutoError::StartsNonZero { name: "x".into() })
    );
}

#[test]
fn a_region_lets_go_of_an_ancilla_declared_outside_each_round() {
    let src = "fn f(i: &mut i64, a: &i64) {
        auto<'round> ancilla x: i64 = 0;
        'round: from i == 0 {
            x += a;
        } loop { i += 1; } until i == 3;
        i -= 3;
    }";
    assert_eq!(check(&expanded(src).unwrap()), Ok(()));
}

#[test]
fn a_region_may_be_a_labeled_block() {
    let src = "fn f(a: &i64) {
        auto<'part> ancilla x: i64 = 0;
        'part: { x += a; }
    }";
    assert_eq!(check(&expanded(src).unwrap()), Ok(()));
}

#[test]
fn a_region_that_is_not_there_is_an_error() {
    assert_eq!(
        expanded("fn f() { auto<'gone> ancilla x: i64 = 0; }"),
        Err(AutoError::UnknownRegion {
            name: "x".into(),
            region: "gone".into()
        })
    );
}

#[test]
fn a_label_on_something_else_is_not_a_region() {
    let src = "fn f(a: &i64) {
        auto<'r> ancilla x: i64 = 0;
        'r: x += a;
    }";
    assert_eq!(
        expanded(src),
        Err(AutoError::NotARegion { region: "r".into() })
    );
}

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

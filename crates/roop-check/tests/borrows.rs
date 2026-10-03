use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("fn f(p: &mut P, a: &mut [i64; 4], i: &mut i64, y: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_update_through_alias() {
    assert_eq!(run("borrow c = a[0] { c += y; }"), Ok(()));
    assert_eq!(run("borrow c = p.re { c += y; } y += 1;"), Ok(()));
}

#[test]
fn rejects_use_of_borrowed_place() {
    assert!(matches!(
        run("borrow c = a[0] { c += a[1]; }"),
        Err(CheckError::BorrowedPlaceUsed { .. })
    ));
}

#[test]
fn rejects_use_of_whole_parent_in_nested_block() {
    assert!(matches!(
        run("borrow c = p.re { if y > 0 { p.re += 1; } fi y > 0; }"),
        Err(CheckError::BorrowedPlaceUsed { .. })
    ));
}

#[test]
fn allows_disjoint_field_while_borrowed() {
    assert_eq!(run("borrow c = p.re { p.im += y; c += y; }"), Ok(()));
}

#[test]
fn rejects_modifying_the_selector() {
    assert!(matches!(
        run("borrow c = a[i] { i += 1; }"),
        Err(CheckError::BorrowIndexModified { var, .. }) if var == "i"
    ));
}

#[test]
fn checks_alias_updates_for_interference() {
    assert!(matches!(
        run("borrow c = a[0] { c += c; }"),
        Err(CheckError::SelfReferentialUpdate { .. })
    ));
}

#[test]
fn ancilla_written_through_alias_is_rejected() {
    assert!(matches!(
        run("ancilla t: i64 = 0 { borrow c = t { c += 1; } }"),
        Err(CheckError::AncillaTouchedInControlFlow { .. })
    ));
}

use roop_check::{CheckError, check};
use roop_syntax::parse;

fn run(members: &str) -> Result<(), CheckError> {
    let src = format!("rev struct P {{ a: i64, b: i64, {members} }}");
    check(&parse(&src).unwrap())
}

#[test]
fn accepts_struct_without_constructors() {
    assert_eq!(run(""), Ok(()));
}

#[test]
fn accepts_unbuild_that_inverts_build_in_reverse() {
    assert_eq!(
        run("build(x: &i64) { self.a += x; self.b += x; }
             unbuild(x: &i64) { self.b -= x; self.a -= x; }"),
        Ok(())
    );
}

#[test]
fn rejects_build_without_unbuild() {
    assert!(matches!(
        run("build(x: &i64) { self.a += x; }"),
        Err(CheckError::UnpairedBuild { .. })
    ));
}

#[test]
fn rejects_unbuild_in_wrong_order() {
    assert!(matches!(
        run("build(x: &i64) { self.a += x; self.b ^= x; }
             unbuild(x: &i64) { self.a -= x; self.b ^= x; }"),
        Err(CheckError::UnbuildNotInverse { .. })
    ));
}

#[test]
fn rejects_unbuild_with_different_parameters() {
    assert!(matches!(
        run("build(x: &i64) { self.a += x; }
             unbuild(y: &i64) { self.a -= y; }"),
        Err(CheckError::UnbuildNotInverse { .. })
    ));
}

#[test]
fn checks_constructor_bodies_for_interference() {
    assert!(matches!(
        run("build(x: &i64) { self.a += self.a; }
             unbuild(x: &i64) { self.a -= self.a; }"),
        Err(CheckError::SelfReferentialUpdate { .. })
    ));
}

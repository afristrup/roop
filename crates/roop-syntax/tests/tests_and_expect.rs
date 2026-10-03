use roop_syntax::{Block, Expr, Item, StmtKind, Type, parse};

fn only_fn(src: &str) -> roop_syntax::FnDef {
    let program = parse(src).unwrap();
    match program.items.into_iter().next() {
        Some(Item::Fn(f)) => f,
        other => panic!("expected a function, got {other:?}"),
    }
}

#[test]
fn a_test_is_a_function_over_its_fixtures() {
    let f = only_fn("test t { n: i64, a: [i64; 3]; n += 4; }");
    assert!(f.test);
    assert_eq!(f.name, "t");
    let names: Vec<_> = f.params.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["n", "a"]);
    assert!(matches!(f.params[0].ty, Type::Ref { mutable: true, .. }));
    assert_eq!(f.body.stmts.len(), 1);
}

#[test]
fn a_test_needs_no_fixtures() {
    let f = only_fn("test t { expect true; }");
    assert!(f.params.is_empty());
    assert_eq!(f.body.stmts.len(), 1);
}

#[test]
fn test_stays_usable_as_a_name() {
    let f = only_fn("fn test(test: &mut i64) { test += 1; }");
    assert!(!f.test);
    assert_eq!(f.name, "test");
}

#[test]
fn expect_is_an_if_that_fails_when_the_condition_does() {
    let f = only_fn("fn f(a: &mut i64) { expect a == 1; }");
    let StmtKind::If {
        then_block,
        else_block,
        exit,
        ..
    } = &f.body.stmts[0].kind
    else {
        panic!("expected an if")
    };
    assert_eq!(then_block.stmts.len() + else_block.stmts.len(), 0);
    assert_eq!(*exit, Expr::Bool(true));
}

#[test]
fn expect_stays_usable_as_a_variable() {
    let f = only_fn("fn f(expect: &mut i64) { expect += 1; expect = 2; }");
    assert_eq!(f.body.stmts.len(), 2);
}

#[test]
fn empty_blocks_default_to_no_span() {
    assert!(Block::default().stmts.is_empty());
}

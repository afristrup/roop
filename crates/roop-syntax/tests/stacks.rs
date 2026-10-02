use roop_syntax::{Expr, Item, StmtKind, Type, parse};

fn body(src: &str) -> Vec<StmtKind> {
    let program = parse(&format!(
        "fn f(x: &mut i64, h: &mut Stack<i64, 8>, failed: &mut bool) {{ {src} }}"
    ))
    .unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    f.body.stmts.iter().map(|s| s.kind.clone()).collect()
}

#[test]
fn parses_the_stack_type() {
    let program = parse("fn f(h: &mut Stack<i64, 8>) { }").unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    let Type::Ref { inner, .. } = &f.params[0].ty else {
        panic!("expected a reference")
    };
    assert_eq!(**inner, Type::Stack(Box::new(Type::Named("i64".into())), 8));
}

#[test]
fn parses_push_pop_and_logged() {
    let stmts = body("push h <- x; pop h -> x; logged h { x = 1; }");
    assert!(matches!(stmts[0], StmtKind::Push { .. }));
    assert!(matches!(stmts[1], StmtKind::Pop { .. }));
    assert!(matches!(stmts[2], StmtKind::Logged { .. }));
}

#[test]
fn parses_an_ancilla_stack_that_starts_empty() {
    let stmts = body("ancilla g: Stack<i64, 4> = empty { push g <- x; pop g -> x; }");
    let StmtKind::Ancilla { init, .. } = &stmts[0] else {
        panic!("expected an ancilla")
    };
    assert_eq!(*init, Expr::Empty);
}

#[test]
fn parses_try_with_and_without_an_outcome() {
    let stmts = body(
        "try { x += 1; } catch_rollback { x -= 1; } -> failed;
         try { x += 1; } catch_rollback { x -= 1; }",
    );
    assert!(matches!(
        &stmts[0],
        StmtKind::Try {
            outcome: Some(_),
            ..
        }
    ));
    assert!(matches!(&stmts[1], StmtKind::Try { outcome: None, .. }));
}

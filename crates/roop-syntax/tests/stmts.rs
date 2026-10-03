use roop_syntax::{Item, StmtKind, parse};

fn first_stmt(body: &str) -> StmtKind {
    let program = parse(&format!("fn f(x: &mut i64, y: &mut i64) {{ {body} }}")).unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    f.body.stmts[0].kind.clone()
}

#[test]
fn parses_updates_and_swap() {
    assert!(matches!(first_stmt("x += y * 2;"), StmtKind::Update { .. }));
    assert!(matches!(first_stmt("x -= 1;"), StmtKind::Update { .. }));
    assert!(matches!(first_stmt("x ^= y;"), StmtKind::Update { .. }));
    assert!(matches!(first_stmt("x <=> y;"), StmtKind::Swap(..)));
}

#[test]
fn parses_if_fi_with_and_without_else() {
    assert!(matches!(
        first_stmt("if x > 0 { y += x; } else { y -= x; } fi y > 0;"),
        StmtKind::If { .. }
    ));
    assert!(matches!(
        first_stmt("if x > 0 { y += x; } fi y > 0;"),
        StmtKind::If { .. }
    ));
}

#[test]
fn parses_from_loop_until() {
    let kind = first_stmt("from x == 0 { y += 1; } loop { x += 1; } until x == 10;");
    assert!(matches!(kind, StmtKind::From { .. }));
}

#[test]
fn parses_ancilla_call_uncall() {
    assert!(matches!(
        first_stmt("ancilla t: i64 = 0 { t += x; t -= x; }"),
        StmtKind::Ancilla { .. }
    ));
    assert!(matches!(first_stmt("call g(x, 1);"), StmtKind::Call { .. }));
    assert!(matches!(
        first_stmt("uncall g(x, 1);"),
        StmtKind::Uncall { .. }
    ));
}

#[test]
fn parses_match_with_exit_assertions() {
    let kind = first_stmt(
        "match x { 0 => { y += 1; } assert y == 1; true => { y -= 1; } assert y != 1; _ => { } assert y == 0; }",
    );
    let StmtKind::Match { arms, .. } = kind else {
        panic!("expected match")
    };
    assert_eq!(arms.len(), 3);
}

#[test]
fn parses_try_catch_rollback() {
    let kind = first_stmt("try { if x > 0 { y += x; } fi y > 0; } catch_rollback { y += 1; }");
    assert!(matches!(kind, StmtKind::Try { .. }));
}

#[test]
fn parses_borrow() {
    let kind = first_stmt("borrow c = x { c += y; }");
    assert!(matches!(kind, StmtKind::Borrow { .. }));
}

#[test]
fn parses_parallel_attribute_with_and_without_target() {
    use roop_syntax::{Attr, Target};
    let src = "fn f(a: &mut [i64; 4], i: &mut i64) {
        #[parallel] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 3;
        #[parallel(metal)] from i == 0 { a[i] += 1; } loop { i += 1; } until i == 3;
    }";
    let program = parse(src).unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    assert_eq!(f.body.stmts[0].attrs, [Attr::Parallel { target: None }]);
    assert_eq!(
        f.body.stmts[1].attrs,
        [Attr::Parallel {
            target: Some(Target::Metal)
        }]
    );
}

#[test]
fn parses_channels_sends_receives_and_concurrent_blocks() {
    use roop_syntax::Attr;
    let src = "fn f(x: &mut i64, y: &mut i64) {
        chan c: i64 {
            #[concurrent] { send c <- x; }
            #[concurrent] { recv c -> y; }
        }
    }";
    let program = parse(src).unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    let StmtKind::Chan { name, body, .. } = &f.body.stmts[0].kind else {
        panic!("expected chan")
    };
    assert_eq!(name, "c");
    assert_eq!(body.stmts.len(), 2);
    assert_eq!(body.stmts[0].attrs, [Attr::Concurrent]);
    assert!(matches!(body.stmts[0].kind, StmtKind::Block(_)));
}

#[test]
fn a_bare_block_is_a_statement() {
    assert!(matches!(first_stmt("{ x += 1; }"), StmtKind::Block(_)));
}

#[test]
fn parses_attributes_in_either_order() {
    let src = "fn f(x: &mut i64) { #[concurrent] #[parallel] { x += 1; } }";
    assert!(parse(src).is_ok());
}

#[test]
fn keep_is_a_statement_of_one_place() {
    let program = parse("fn f(a: &mut [i64; 2]) { keep a[1]; }").unwrap();
    let roop_syntax::Item::Fn(f) = &program.items[0] else {
        panic!("not a function");
    };
    assert!(matches!(
        f.body.stmts[0].kind,
        roop_syntax::StmtKind::Keep(_)
    ));
}

#[test]
fn an_auto_names_its_region_and_a_statement_its_label() {
    let src = "fn f(i: &mut i64) { auto<'r> ancilla x: i64 = 0; 'r: from i == 0 { x += 1; } until i == 1; }";
    let program = parse(src).unwrap();
    let roop_syntax::Item::Fn(f) = &program.items[0] else {
        panic!("not a function");
    };
    let roop_syntax::StmtKind::Ancilla { body, .. } = &f.body.stmts[0].kind else {
        panic!("not an ancilla");
    };
    assert_eq!(
        f.body.stmts[0].attrs,
        [roop_syntax::Attr::Auto {
            region: Some("r".into())
        }]
    );
    assert_eq!(body.stmts[0].attrs, [roop_syntax::Attr::Label("r".into())]);
}

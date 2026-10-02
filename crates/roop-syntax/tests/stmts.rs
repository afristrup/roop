use roop_syntax::{Item, StmtKind, parse};

fn first_stmt(body: &str) -> StmtKind {
    let program = parse(&format!("rev fn f(x: &mut i64, y: &mut i64) {{ {body} }}")).unwrap();
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

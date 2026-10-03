use roop_syntax::{Item, OverwriteOp, StmtKind, UpdateOp, parse};

fn first_stmt(body: &str) -> StmtKind {
    let program = parse(&format!(
        "irrev fn f(x: &mut i64, y: &mut i64) {{ {body} }}"
    ))
    .unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected fn")
    };
    f.body.stmts[0].kind.clone()
}

#[test]
fn irrev_marks_a_function_and_plain_fn_is_reversible() {
    let program = parse("irrev fn a() { } fn b() { }").unwrap();
    let flags: Vec<bool> = program
        .items
        .iter()
        .map(|item| match item {
            Item::Fn(f) => f.irreversible,
            _ => panic!("expected fn"),
        })
        .collect();
    assert_eq!(flags, [true, false]);
}

#[test]
fn the_rev_keyword_is_gone() {
    assert!(parse("rev fn f() { }").is_err());
    assert!(parse("rev struct S { a: i64 }").is_err());
}

#[test]
fn parses_overwrite_statements() {
    for (src, op) in [
        ("x = y + 1;", OverwriteOp::Assign),
        ("x %= 2;", OverwriteOp::Rem),
    ] {
        match first_stmt(src) {
            StmtKind::Overwrite { op: got, .. } => assert_eq!(got, op, "{src}"),
            other => panic!("{src}: {other:?}"),
        }
    }
}

#[test]
fn parses_irrev_blocks() {
    assert!(matches!(first_stmt("irrev { x = 0; }"), StmtKind::Irrev(_)));
}

#[test]
fn star_and_slash_are_reversible_updates() {
    for (src, want) in [("x *= 2;", UpdateOp::Mul), ("x /= 2;", UpdateOp::Div)] {
        match first_stmt(src) {
            StmtKind::Update { op, .. } => assert_eq!(op, want, "{src}"),
            other => panic!("{src}: {other:?}"),
        }
    }
}

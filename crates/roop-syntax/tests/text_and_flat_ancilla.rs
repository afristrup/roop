use roop_syntax::{Expr, Item, StmtKind, Type, parse};

fn body(src: &str) -> Vec<roop_syntax::Stmt> {
    match parse(src).unwrap().items.into_iter().next() {
        Some(Item::Fn(f)) => f.body.stmts,
        other => panic!("expected a function, got {other:?}"),
    }
}

#[test]
fn a_string_literal_is_its_bytes() {
    let stmts = body("fn f() { call g(\"a\\n\\x41\\\"\"); }");
    let StmtKind::Call { args, .. } = &stmts[0].kind else {
        panic!("expected a call")
    };
    assert_eq!(args[0], Expr::Str(vec![b'a', b'\n', 0x41, b'"']));
}

#[test]
fn a_byte_literal_is_one_byte() {
    let stmts = body("fn f(c: &mut u8) { c += b'a'; c += b'\\n'; c += b'\\''; }");
    let values: Vec<Expr> = stmts
        .iter()
        .map(|s| match &s.kind {
            StmtKind::Update { value, .. } => value.clone(),
            _ => panic!("expected an update"),
        })
        .collect();
    assert_eq!(
        values,
        [Expr::Byte(b'a'), Expr::Byte(b'\n'), Expr::Byte(b'\'')]
    );
}

#[test]
fn a_byte_literal_of_two_bytes_is_refused() {
    assert!(parse("fn f(c: &mut u8) { c += b'ab'; }").is_err());
    assert!(parse("fn f(c: &mut u8) { c += b'\\x4'; }").is_err());
}

#[test]
fn slashes_in_a_string_are_not_a_comment() {
    let program = parse("fn f() { call g(\"http://x\"); } // real").unwrap();
    assert_eq!(program.items.len(), 1);
    let comments = roop_syntax::comments("fn f() { call g(\"http://x\"); } // real\n");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].text, "// real");
}

#[test]
fn a_cast_binds_tighter_than_arithmetic_and_looser_than_negation() {
    let stmts = body("fn f(x: &mut i64, y: &i64) { x += -y as i64 + y as i64 * 2; }");
    let StmtKind::Update { value, .. } = &stmts[0].kind else {
        panic!("expected an update")
    };
    let Expr::Binary(left, _, right) = value else {
        panic!("expected a sum")
    };
    assert!(matches!(&**left, Expr::Cast(inner, _) if matches!(**inner, Expr::Unary(..))));
    assert!(matches!(&**right, Expr::Binary(l, _, _) if matches!(**l, Expr::Cast(..))));
}

#[test]
fn a_cast_names_a_type() {
    let stmts = body("fn f(x: &mut f64, n: &i64) { x += n as f64; }");
    let StmtKind::Update { value, .. } = &stmts[0].kind else {
        panic!("expected an update")
    };
    assert!(matches!(value, Expr::Cast(_, Type::Named(t)) if t == "f64"));
}

#[test]
fn a_declaration_lasts_to_the_end_of_its_block() {
    let stmts = body(
        "fn f(a: &mut i64) { a += 1; ancilla i: i64 = 0; a += i; ancilla j: i64 = 0; a += j; }",
    );
    assert_eq!(stmts.len(), 2);
    let StmtKind::Ancilla { name, body, .. } = &stmts[1].kind else {
        panic!("expected an ancilla")
    };
    assert_eq!(name, "i");
    assert_eq!(body.stmts.len(), 2);
    let StmtKind::Ancilla { name, body, .. } = &body.stmts[1].kind else {
        panic!("expected a second ancilla")
    };
    assert_eq!(name, "j");
    assert_eq!(body.stmts.len(), 1);
}

#[test]
fn a_declaration_and_the_braced_form_are_the_same_program() {
    let flat = body("fn f(a: &mut i64) { ancilla i: i64 = 0; a += i; }");
    let braced = body("fn f(a: &mut i64) { ancilla i: i64 = 0 { a += i; } }");
    let strip = |s: &roop_syntax::Stmt| match &s.kind {
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => (name.clone(), ty.clone(), init.clone(), body.stmts.len()),
        _ => panic!("expected an ancilla"),
    };
    assert_eq!(strip(&flat[0]), strip(&braced[0]));
}

#[test]
fn a_declaration_with_nothing_after_it_has_an_empty_body() {
    let stmts = body("fn f() { ancilla i: i64 = 0; }");
    let StmtKind::Ancilla { body, .. } = &stmts[0].kind else {
        panic!("expected an ancilla")
    };
    assert!(body.stmts.is_empty());
}

#[test]
fn an_extern_function_has_a_signature_and_no_body() {
    let program = parse("pub extern fn roop_write<N>(fd: &i64, buf: &[u8; N]);").unwrap();
    let Item::Fn(f) = &program.items[0] else {
        panic!("expected a function")
    };
    assert!(f.external && f.irreversible && f.public);
    assert_eq!(f.generics, ["N"]);
    assert_eq!(f.params.len(), 2);
    assert!(f.body.stmts.is_empty());
}

#[test]
fn extern_is_a_keyword() {
    assert!(parse("fn extern() { }").is_err());
}

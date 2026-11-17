use roop_hybrid::{Expr, HybridError, Kind, Stmt, check, compile, parse};

fn error(text: &str) -> HybridError {
    compile(text, true).unwrap_err()
}

#[test]
fn do_while_unrolls_its_first_round() {
    let stmts = parse("bit c; do c := ¬c while c").unwrap();
    assert!(matches!(&stmts[1], Stmt::Assign { name, .. } if name == "c"));
    assert!(matches!(&stmts[2], Stmt::While { body, .. } if body.len() == 1));
}

#[test]
fn negation_binds_looser_than_comparison() {
    let stmts = parse("int x; bit c; c := !x <= 3 && c").unwrap();
    let Stmt::Assign { value, .. } = &stmts[2] else {
        panic!()
    };
    assert!(matches!(value, Expr::And(a, _) if matches!(**a, Expr::Not(_))));
}

#[test]
fn powers_group_to_the_right() {
    let stmts = parse("int x; x := 2 ^ 3 ^ 2").unwrap();
    let Stmt::Assign { value, .. } = &stmts[1] else {
        panic!()
    };
    assert!(matches!(value, Expr::Pow(_, b) if matches!(**b, Expr::Pow(..))));
}

#[test]
fn the_signature_follows_declaration_order() {
    let sig =
        check(&parse("qubit q; bit c; int x; if c then skip else skip end").unwrap()).unwrap();
    assert_eq!(
        sig.entries,
        vec![
            ("q".into(), Kind::Qubit),
            ("c".into(), Kind::Bit),
            ("x".into(), Kind::Int)
        ]
    );
}

#[test]
fn memory_must_be_allocated_before_use() {
    assert_eq!(
        error("H(q)"),
        HybridError::Undeclared {
            line: 1,
            name: "q".into()
        }
    );
}

#[test]
fn memory_is_allocated_once() {
    assert_eq!(
        error("qubit q;\nqubit q"),
        HybridError::Redeclared {
            line: 2,
            name: "q".into()
        }
    );
}

#[test]
fn a_gate_cannot_use_a_qubit_twice() {
    assert_eq!(
        error("qubit q; CNOT(q, q)"),
        HybridError::SharedQubit {
            line: 1,
            qubit: "q".into()
        }
    );
}

#[test]
fn loops_cannot_allocate() {
    let e = error("bit c; while c do qubit q done");
    assert_eq!(
        e,
        HybridError::LoopAllocation {
            line: 1,
            name: "q".into()
        }
    );
}

#[test]
fn branches_allocate_alike() {
    let e = error("bit c; if c then int x else skip end");
    assert_eq!(
        e,
        HybridError::Branches {
            line: 1,
            names: vec!["x".into()]
        }
    );
    compile("bit c; if c then int x else int x end; x := x + 1", true).unwrap();
}

#[test]
fn a_qubit_is_not_a_classical_value() {
    let e = error("qubit q; bit c; c := q");
    assert_eq!(
        e,
        HybridError::Kind {
            line: 1,
            name: "q".into(),
            expected: Kind::Bit,
            found: Kind::Qubit
        }
    );
}

#[test]
fn bits_take_only_zero_and_one() {
    assert_eq!(
        error("bit c; c := 2"),
        HybridError::Type {
            line: 1,
            expected: Kind::Bit
        }
    );
    assert_eq!(
        error("int x; bit c; c := x"),
        HybridError::Kind {
            line: 1,
            name: "x".into(),
            expected: Kind::Bit,
            found: Kind::Int
        }
    );
}

#[test]
fn rotations_stop_at_r62() {
    assert_eq!(
        error("qubit q; R(q, 63)"),
        HybridError::Rotation { line: 1, k: 63 }
    );
    assert_eq!(
        error("qubit q; R70(q)"),
        HybridError::Rotation { line: 1, k: 70 }
    );
}

#[test]
fn syntax_errors_name_the_line() {
    let e = error("qubit q;\nH(q) H(q)");
    assert_eq!(
        e,
        HybridError::Parse {
            line: 2,
            expected: "`;`".into(),
            found: "`H`".into()
        }
    );
}

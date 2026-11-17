use roop_check::{CheckError, check};
use roop_syntax::parse;

const GATES: &str = "
    fn h(q: &mut qubit) { }
    fn cx(c: &mut qubit, t: &mut qubit) { }
";

fn run(body: &str) -> Result<(), CheckError> {
    let src = format!("{GATES} fn f(a: &mut qubit, qs: &mut [qubit; 2], n: &mut i64) {{ {body} }}");
    check(&parse(&src).unwrap())
}

fn misuse(result: Result<(), CheckError>, name: &str) -> bool {
    matches!(result, Err(CheckError::QubitAsValue { var, .. }) if var == name)
}

#[test]
fn gates_take_qubits_and_array_elements() {
    assert_eq!(
        run("call h(a); call cx(qs[0], qs[1]); uncall h(qs[n]);"),
        Ok(())
    );
}

#[test]
fn a_qubit_ancilla_is_left_to_the_device_to_check() {
    assert_eq!(
        run("ancilla b: qubit = 0 { call cx(a, b); call h(qs[0]); uncall cx(a, b); }"),
        Ok(())
    );
    assert_eq!(
        run("ancilla b: qubit = 0 { call cx(a, b); call cx(b, qs[0]); uncall cx(a, b); }"),
        Ok(())
    );
}

#[test]
fn qubits_are_not_values() {
    assert!(misuse(run("n += a;"), "a"));
    assert!(misuse(run("if a == 0 { } else { } fi n == 0;"), "a"));
    assert!(misuse(run("call h(qs[a]);"), "a"));
    assert!(misuse(run("a += 1;"), "a"));
    assert!(misuse(run("a <=> n;"), "a"));
    assert!(misuse(
        run("ancilla m: i64 = 0 { m += qs[0]; m -= qs[0]; }"),
        "qs"
    ));
}

#[test]
fn qubits_swap_with_qubits() {
    assert_eq!(run("a <=> qs[1];"), Ok(()));
}

#[test]
fn a_qubit_ancilla_starts_at_zero() {
    assert!(matches!(
        run("ancilla b: qubit = 1 { }"),
        Err(CheckError::QubitAncillaStart { .. })
    ));
}

#[test]
fn a_shadowing_integer_is_not_a_qubit() {
    assert_eq!(run("ancilla a: i64 = 0 { a += n; a -= n; }"), Ok(()));
}

#[test]
fn qubits_go_by_mutable_reference() {
    let src = "fn f(q: &qubit) { }";
    assert!(matches!(
        check(&parse(src).unwrap()),
        Err(CheckError::QubitParam { .. })
    ));
}

#[test]
fn structs_hold_no_qubits() {
    let src = "struct P { q: qubit }";
    assert!(matches!(
        check(&parse(src).unwrap()),
        Err(CheckError::QubitInStruct { .. })
    ));
}

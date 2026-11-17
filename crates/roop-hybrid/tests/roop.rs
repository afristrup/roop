use roop_hybrid::{Gate, HybridError, Op, Turns, emit_qasm, lower_roop, optimize};
use roop_syntax::parse;

const DEVICE: &str = "
    extern world fn roop_q_h(q: &mut qubit);
    extern world fn roop_q_x(q: &mut qubit);
    extern world fn roop_q_t(q: &mut qubit);
    extern world fn roop_q_r(q: &mut qubit, k: &i64);
    extern world fn roop_q_cx(c: &mut qubit, t: &mut qubit);
    extern fn roop_q_measure(q: &mut qubit, c: &mut bool);
";

fn lower(src: &str) -> Result<Vec<Op>, HybridError> {
    let program = parse(&format!("{DEVICE}{src}")).unwrap();
    roop_check::check(&program).unwrap();
    lower_roop(&program).map(|p| p.ops)
}

fn kernels(ops: &[Op]) -> Vec<Vec<Gate>> {
    ops.iter()
        .filter_map(|op| match op {
            Op::Kernel(g) => Some(g.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn uncall_inlines_the_adjoint_in_reverse() {
    let ops = lower(
        "fn f(q: &mut qubit) { call roop_q_h(q); call roop_q_t(q); }
         fn main() { ancilla q: qubit = 0; uncall f(q); }",
    )
    .unwrap();
    assert_eq!(
        kernels(&ops),
        vec![vec![Gate::Phase(0, Turns::new(7, 3)), Gate::H(0)]]
    );
}

#[test]
fn a_circuit_and_its_uncall_cancel() {
    let program = parse(&format!(
        "{DEVICE}
         fn bell(a: &mut qubit, b: &mut qubit) {{ call roop_q_h(a); call roop_q_cx(a, b); }}
         fn main() {{ ancilla a: qubit = 0; ancilla b: qubit = 0; call bell(a, b); uncall bell(a, b); }}"
    ))
    .unwrap();
    assert!(optimize(lower_roop(&program).unwrap()).ops.is_empty());
}

#[test]
fn rotations_take_their_k_from_the_call() {
    let ops = lower("fn main() { ancilla q: qubit = 0; call roop_q_r(q, 4); }").unwrap();
    assert_eq!(kernels(&ops), vec![vec![Gate::Phase(0, Turns::new(1, 4))]]);
}

#[test]
fn qubit_arrays_need_known_indices() {
    let ok =
        lower("fn main() { ancilla qs: [qubit; 2] = 0; call roop_q_cx(qs[0], qs[1]); }").unwrap();
    assert_eq!(kernels(&ok), vec![vec![Gate::Cnot(0, 1)]]);
    let err = lower(
        "irrev fn main() { ancilla qs: [qubit; 2] = 0; ancilla i: i64 = 0; i += 1; call roop_q_x(qs[i]); }",
    );
    assert!(
        matches!(err, Err(HybridError::Unsupported { what, .. }) if what.contains("not known"))
    );
}

#[test]
fn a_roop_loop_runs_its_body_before_the_test() {
    let ops = lower(
        "irrev fn main() {
             ancilla q: qubit = 0;
             auto ancilla c: bool = false;
             ancilla n: i64 = 0;
             from n == 0 { c = false; call roop_q_h(q); call roop_q_measure(q, c); } loop { n += 1; } until c;
         }",
    )
    .unwrap();
    assert!(
        matches!(ops.iter().find(|op| matches!(op, Op::While { .. })), Some(Op::While { body, .. }) if body.len() == 4)
    );
    assert!(matches!(ops.iter().position(|op| matches!(op, Op::Measure { .. })), Some(i) if i < 6));
}

#[test]
fn qubit_qasm_names_every_ancilla() {
    let program = parse(&format!(
        "{DEVICE} irrev fn main() {{ ancilla a: qubit = 0; ancilla m: bool = false; call roop_q_h(a); call roop_q_measure(a, m); keep m; }}"
    ))
    .unwrap();
    let qasm = emit_qasm(&optimize(lower_roop(&program).unwrap()));
    assert!(qasm.contains("// q[0] is a\n"), "{qasm}");
    assert!(qasm.contains("c[0] = measure q[0];"), "{qasm}");
    assert!(
        qasm.contains("c[1] = 1;") || qasm.contains("c[1] = 0;"),
        "{qasm}"
    );
}

#[test]
fn constructs_without_a_circuit_are_refused() {
    let err = lower("fn main() { ancilla s: Stack<i64, 2> = empty { } }");
    assert!(matches!(err, Err(HybridError::Unsupported { .. })));
}

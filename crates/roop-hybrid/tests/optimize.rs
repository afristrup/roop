use roop_hybrid::{Gate, Op, Turns, compile, emit_qasm, simplify_kernel};

fn kernels(text: &str) -> Vec<Vec<Gate>> {
    let program = compile(text, true).unwrap();
    program
        .ops
        .into_iter()
        .filter_map(|op| match op {
            Op::Kernel(gates) => Some(gates),
            _ => None,
        })
        .collect()
}

#[test]
fn self_inverse_pairs_cancel_through_commuting_gates() {
    let gates = vec![Gate::Cnot(0, 1), Gate::X(1), Gate::Cnot(0, 1), Gate::X(1)];
    assert!(simplify_kernel(gates).is_empty());
}

#[test]
fn phases_add_through_controls() {
    let t = Turns::new(1, 3);
    let gates = vec![Gate::Phase(0, t), Gate::Cnot(0, 1), Gate::Phase(0, t)];
    assert_eq!(
        simplify_kernel(gates),
        vec![Gate::Phase(0, Turns::new(1, 2)), Gate::Cnot(0, 1)]
    );
}

#[test]
fn hadamard_blocks_motion() {
    let gates = vec![Gate::X(0), Gate::H(0), Gate::X(0)];
    assert_eq!(simplify_kernel(gates.clone()), gates);
}

#[test]
fn kernels_fuse_across_classical_work() {
    let k = kernels("qubit q; qubit r; bit c; int x; H(q); x := x + 1; c := measure q; H(r); T(q)");
    assert_eq!(
        k,
        vec![
            vec![Gate::H(0), Gate::H(1)],
            vec![Gate::Phase(0, Turns::new(1, 3))]
        ]
    );
}

#[test]
fn the_cancel_example_leaves_one_gate() {
    let k = kernels(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/programs/cancel.hyq"))
            .unwrap(),
    );
    assert_eq!(k, vec![vec![Gate::H(0)]]);
}

#[test]
fn qasm_has_registers_control_and_kernels() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/programs/coin_toss.hyq"
    ))
    .unwrap();
    let qasm = emit_qasm(&compile(&text, true).unwrap());
    let expected = "OPENQASM 3.0;
include \"stdgates.inc\";

// q[0] is q
// c[0] is c
qubit[1] q;
reset q;
bit[1] c = \"0\";
int[64] v_x = 0;

// kernel 0
h q[0];
c[0] = measure q[0];
v_x = (v_x + 1);
while (!(c[0] == 1)) {
    // kernel 1
    h q[0];
    c[0] = measure q[0];
    v_x = (v_x + 1);
}
";
    assert_eq!(qasm, expected);
}

#[test]
fn odd_phases_keep_their_t() {
    let qasm = emit_qasm(&compile("qubit q; T(q); T(q); T(q); R(q, 4)", true).unwrap());
    assert!(qasm.contains("p(7 * pi / 8) q[0];"), "{qasm}");
    let qasm = emit_qasm(&compile("qubit q; T(q); T(q); T(q)", true).unwrap());
    assert!(qasm.contains("s q[0]; t q[0];"), "{qasm}");
}

#[test]
fn a_bit_measured_over_is_not_set() {
    let program = compile("qubit q; bit c; c := 1; H(q); c := measure q", true).unwrap();
    assert!(!program.ops.iter().any(|op| matches!(op, Op::SetBit { .. })));
}

#[test]
fn a_store_that_is_read_stays() {
    let program = compile("bit c; bit d; c := 1; d := c; c := 0", true).unwrap();
    assert_eq!(program.ops.len(), 3);
}

#[test]
fn the_last_store_of_the_program_stays() {
    let program = compile("int x; x := 1; x := 2", true).unwrap();
    assert_eq!(
        program.ops,
        vec![Op::SetInt {
            reg: 0,
            value: roop_hybrid::Int::Lit(2)
        }]
    );
}

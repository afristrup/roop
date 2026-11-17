mod support;

use std::path::{Path, PathBuf};
use std::process::Output;
use support::{project, roop, stderr};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../roop")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn example(name: &str) -> Output {
    roop(
        &library(),
        &["run", &format!("examples/quantum/{name}.roop")],
    )
}

fn scratch(name: &str, source: &str) -> PathBuf {
    let config = format!("[modules]\nstd = \"{}\"\n", library().join("std").display());
    project(&format!("quantum-{name}"), &config, source)
}

#[test]
fn a_bell_pair_is_made_and_unmade() {
    let out = example("bell");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "entangled\n");
}

#[test]
fn deutsch_tells_constant_from_balanced_in_one_query() {
    let out = example("deutsch");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "constant\nbalanced\n");
}

#[test]
fn teleportation_moves_a_state() {
    let out = example("teleport");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "teleported |1>\n");
}

#[test]
fn a_coin_toss_loop_ends() {
    let out = example("coin_toss");
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).starts_with("tosses: "), "{}", stdout(&out));
}

#[test]
fn quantum_tests_run_forward_and_backward() {
    let out = roop(&library(), &["test", "tests/quantum.roop"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("5 passed, 0 failed"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn an_ancilla_qubit_left_in_superposition_stops_the_program() {
    let dir = scratch(
        "leak",
        "use std::quantum::*;\nfn main() { ancilla a: qubit = 0; call h(a); }",
    );
    let out = roop(&dir, &["run", "prog.roop"]);
    assert_eq!(out.status.code(), Some(70), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("probability 5.000e-1 of being 1"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_qubit_cannot_be_read() {
    let src = "use std::quantum::*;\nfn main() { ancilla a: qubit = 0; ancilla k: i64 = 0; k += a; k -= a; }";
    let out = roop(&scratch("copy", src), &["run", "prog.roop"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("`a` is a qubit, which cannot be read"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn measuring_needs_irrev() {
    let src = "use std::quantum::*;\nfn f(q: &mut qubit, c: &mut bool) { call measure(q, c); }";
    let out = roop(
        &scratch("measure", src),
        &["build", "prog.roop", "--emit", "ir"],
    );
    assert!(!out.status.success(), "{}", stderr(&out));
}

#[test]
fn a_failed_try_takes_its_gates_back() {
    let src = "use std::io::*;\nuse std::quantum::*;\nfn main() {\n    ancilla q: qubit = 0;\n    auto ancilla failed: bool = false;\n    try { call h(q); call t(q); expect false; } catch_rollback { call println(\"rolled back\"); } -> failed;\n}";
    let out = roop(&scratch("rollback", src), &["run", "prog.roop"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "rolled back\n");
}

#[test]
fn keep_hands_a_qubit_to_the_world() {
    let src = "use std::quantum::*;\nfn main() { auto ancilla q: qubit = 0; call h(q); }";
    let out = roop(&scratch("keep", src), &["run", "prog.roop"]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn one_qubit_is_not_two_arguments() {
    let src = "use std::quantum::*;\nfn main() { ancilla q: qubit = 0; call cx(q, q); }";
    let out = roop(&scratch("alias", src), &["run", "prog.roop"]);
    assert!(
        stderr(&out).contains("also appears in another argument"),
        "{}",
        stderr(&out)
    );
}

fn hybrid(name: &str, emit: &str) -> String {
    let path = format!("examples/quantum/{name}.roop");
    let out = roop(&library(), &["hybrid", &path, "--emit", emit]);
    assert!(out.status.success(), "{}", stderr(&out));
    stdout(&out)
}

#[test]
fn roop_hybrid_compiles_a_roop_program_to_qasm() {
    let qasm = hybrid("teleport", "qasm");
    assert!(qasm.starts_with("OPENQASM 3.0;"), "{qasm}");
    assert!(qasm.contains("= measure q["), "{qasm}");
    assert!(qasm.contains("if ((c["), "{qasm}");
}

#[test]
fn teleportation_always_arrives() {
    assert!(hybrid("teleport", "report").contains("P(seen_kept = 1) = 1.000000000"));
}

#[test]
fn deutsch_is_exact() {
    let report = hybrid("deutsch", "report");
    assert!(
        report.contains("P(found_kept = 1) = 0.000000000"),
        "{report}"
    );
    assert!(
        report.contains("P(found_1_kept = 1) = 1.000000000"),
        "{report}"
    );
}

#[test]
fn the_coin_toss_takes_one_extra_toss_on_average() {
    assert!(hybrid("coin_toss", "report").contains("E[n_kept] = 1.000000000"));
}

#[test]
fn a_bell_pair_and_its_uncall_compile_to_nothing() {
    assert!(!hybrid("bell", "qasm").contains("// kernel"));
}

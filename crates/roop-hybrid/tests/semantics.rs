mod support;

use roop_hybrid::{Kind, Layout, Limits, Store, compile, execute, expectation};
use support::{close, program, run};

fn int(layout: &Layout, name: &str) -> impl Fn(&Store) -> f64 {
    let i = layout.index(Kind::Int, name);
    move |s| s.ints[i] as f64
}

#[test]
fn coin_toss_needs_two_tosses_on_average() {
    let p = program("coin_toss", true);
    let outcome = execute(&p, &Limits::default()).unwrap();
    assert!(close(outcome.state.trace(), 1.0));
    assert!(close(expectation(&outcome.state, int(&p.layout, "x")), 2.0));
    for (store, rho) in &outcome.state.parts {
        let x = store.ints[0];
        assert!(close(rho.trace(), 0.5f64.powi(x as i32)), "P(x = {x})");
    }
}

#[test]
fn synth_succeeds_after_four_thirds_rounds() {
    let p = program("synth", true);
    let outcome = execute(&p, &Limits::default()).unwrap();
    assert!(close(outcome.state.trace(), 1.0));
    assert!(close(
        expectation(&outcome.state, int(&p.layout, "x")),
        4.0 / 3.0
    ));
    assert!(close(outcome.costs.gates["t"], 2.0 * 4.0 / 3.0));
}

#[test]
fn synth_applies_its_unitary_to_the_data_qubit() {
    let outcome = run("synth", true);
    let rho: Vec<_> = outcome.state.parts.values().cloned().collect();
    let mut total = rho[0].clone();
    rho[1..].iter().for_each(|r| total.add(r));
    // q started in |0⟩, U|0⟩ = (|0⟩ + i√2|1⟩)/√3, and q1 is back in |0⟩.
    let dim = total.dim();
    assert!(close(total.trace(), 1.0));
    assert!(close(total.entries[0].re, 1.0 / 3.0));
    assert!(close(total.entries[dim + 1].re, 2.0 / 3.0));
    assert!(close(total.entries[1].im.abs(), 2f64.sqrt() / 3.0));
}

#[test]
fn nested_loops_terminate_almost_surely() {
    let p = program("nested", true);
    let outcome = execute(&p, &Limits::default()).unwrap();
    assert!(outcome.state.trace() > 1.0 - 1e-9);
    assert!(close(
        expectation(&outcome.state, int(&p.layout, "x1"))
            / expectation(&outcome.state, int(&p.layout, "x2")),
        2.0
    ));
}

#[test]
fn optimizing_keeps_the_semantics() {
    for name in ["coin_toss", "synth", "nested", "cancel"] {
        let (a, b) = (run(name, false), run(name, true));
        assert_eq!(a.state.parts.len(), b.state.parts.len(), "{name}");
        for ((sa, ra), (sb, rb)) in a.state.parts.iter().zip(&b.state.parts) {
            assert_eq!(sa, sb, "{name}");
            for (x, y) in ra.entries.iter().zip(&rb.entries) {
                assert!(close(x.re, y.re) && close(x.im, y.im), "{name}");
            }
        }
    }
}

#[test]
fn a_loop_that_never_ends_is_reported_as_running() {
    let p = compile("bit c; while !c do skip done", true).unwrap();
    let limits = Limits {
        iterations: 50,
        ..Limits::default()
    };
    let outcome = execute(&p, &limits).unwrap();
    assert!(close(outcome.state.trace(), 0.0));
    assert!(close(outcome.truncated, 1.0));
}

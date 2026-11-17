use crate::{HybridProgram, Outcome, expectation};

/// A resource report: termination, the mean of each classical variable on
/// termination, and the expected costs.
pub fn report(program: &HybridProgram, outcome: &Outcome) -> String {
    let layout = &program.layout;
    let state = &outcome.state;
    let mut out = format!(
        "terminated with probability {:.9}\nstill running when unrolling stopped: {:.3e}\n",
        state.trace(),
        outcome.truncated
    );
    if state.trace() > 0.0 {
        out.push_str("on termination:\n");
        for (i, name) in layout.bits.iter().enumerate() {
            let p = expectation(state, |s| f64::from(u8::from(s.bits[i])));
            out.push_str(&format!("  P({name} = 1) = {p:.9}\n"));
        }
        for (i, name) in layout.ints.iter().enumerate() {
            out.push_str(&format!(
                "  E[{name}] = {:.9}\n",
                expectation(state, |s| s.ints[i] as f64)
            ));
        }
    }
    let costs = &outcome.costs;
    out.push_str(&format!(
        "expected costs:\n  kernels {:.9}\n  measurements {:.9}\n  loop iterations {:.9}\n",
        costs.kernels, costs.measurements, costs.iterations
    ));
    for (gate, n) in &costs.gates {
        out.push_str(&format!("  {gate} {n:.9}\n"));
    }
    out
}

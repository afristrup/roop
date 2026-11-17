use crate::Gate;

/// The OpenQASM name of a gate; phases that are not Clifford+T are `p`.
pub fn gate_label(gate: &Gate) -> &'static str {
    match gate {
        Gate::X(_) => "x",
        Gate::H(_) => "h",
        Gate::Cnot(..) => "cx",
        Gate::Phase(_, t) => match (t.num, t.log) {
            (1, 1) => "z",
            (1, 2) => "s",
            (3, 2) => "sdg",
            (1, 3) | (3, 3) | (5, 3) | (7, 3) => "t",
            _ => "p",
        },
    }
}

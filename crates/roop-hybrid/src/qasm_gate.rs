use crate::Gate;

/// One gate as OpenQASM 3 on the register `q`. Odd eighths of a turn are a T
/// or T† after a Clifford phase, so the T count stays visible.
pub fn qasm_gate(gate: &Gate) -> String {
    match *gate {
        Gate::X(q) => format!("x q[{q}];"),
        Gate::H(q) => format!("h q[{q}];"),
        Gate::Cnot(c, t) => format!("cx q[{c}], q[{t}];"),
        Gate::Phase(q, t) => match (t.num, t.log) {
            (1, 1) => format!("z q[{q}];"),
            (1, 2) => format!("s q[{q}];"),
            (3, 2) => format!("sdg q[{q}];"),
            (1, 3) => format!("t q[{q}];"),
            (3, 3) => format!("s q[{q}]; t q[{q}];"),
            (5, 3) => format!("z q[{q}]; t q[{q}];"),
            (7, 3) => format!("tdg q[{q}];"),
            (num, log) => format!("p({num} * pi / {}) q[{q}];", 1u64 << (log - 1)),
        },
    }
}

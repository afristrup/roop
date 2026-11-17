use crate::{Gate, Turns};

/// The gates of an extern `roop_q_*` call, backward when `inverse`: the
/// adjoint. Y is X·Z up to a global phase.
pub fn roop_gate(name: &str, qubits: &[usize], k: Option<i64>, inverse: bool) -> Option<Vec<Gate>> {
    let phase = |num: u64, log: u32| {
        let t = Turns::new(num, log);
        let t = if inverse {
            Turns::new((1u64 << t.log) - t.num, t.log)
        } else {
            t
        };
        Gate::Phase(qubits[0], t)
    };
    Some(match name {
        "roop_q_h" => vec![Gate::H(qubits[0])],
        "roop_q_x" => vec![Gate::X(qubits[0])],
        "roop_q_y" => vec![Gate::Phase(qubits[0], Turns::new(1, 1)), Gate::X(qubits[0])],
        "roop_q_z" => vec![phase(1, 1)],
        "roop_q_s" => vec![phase(1, 2)],
        "roop_q_t" => vec![phase(1, 3)],
        "roop_q_r" => vec![phase(1, u32::try_from(k?).ok().filter(|&k| k <= 62)?)],
        "roop_q_cx" => vec![Gate::Cnot(qubits[0], qubits[1])],
        _ => return None,
    })
}

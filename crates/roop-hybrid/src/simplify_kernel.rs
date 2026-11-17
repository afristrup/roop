use crate::{Gate, combine, commutes};

/// Each gate moves back past the gates it commutes with until it meets one it
/// combines with, so `H T X T H`... chains shrink as far as these rules allow.
pub fn simplify_kernel(gates: Vec<Gate>) -> Vec<Gate> {
    let mut out: Vec<Gate> = Vec::new();
    for gate in gates {
        if matches!(gate, Gate::Phase(_, t) if t.is_zero()) {
            continue;
        }
        let mut merged = false;
        for j in (0..out.len()).rev() {
            if let Some(result) = combine(&out[j], &gate) {
                out.splice(j..=j, result);
                merged = true;
                break;
            }
            if !commutes(&out[j], &gate) {
                break;
            }
        }
        if !merged {
            out.push(gate);
        }
    }
    out
}

use crate::Gate;

/// `b` right after `a` as fewer gates: self-inverse pairs cancel and phases on
/// one qubit add. `None` when the pair does not simplify.
pub fn combine(a: &Gate, b: &Gate) -> Option<Vec<Gate>> {
    match (*a, *b) {
        (Gate::X(p), Gate::X(q)) | (Gate::H(p), Gate::H(q)) if p == q => Some(Vec::new()),
        (Gate::Cnot(c, t), Gate::Cnot(d, u)) if (c, t) == (d, u) => Some(Vec::new()),
        (Gate::Phase(p, s), Gate::Phase(q, t)) if p == q => {
            let sum = s + t;
            Some(if sum.is_zero() {
                Vec::new()
            } else {
                vec![Gate::Phase(p, sum)]
            })
        }
        _ => None,
    }
}

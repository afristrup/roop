/// The unitaries of HQbricks∞, Clifford+R_k, with `S` and `T` for R_2 and R_3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateName {
    X,
    Z,
    H,
    S,
    T,
    Cnot,
    R(u32),
}

impl GateName {
    pub fn arity(self) -> usize {
        match self {
            Self::Cnot => 2,
            _ => 1,
        }
    }
}

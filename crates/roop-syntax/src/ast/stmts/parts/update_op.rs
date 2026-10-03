#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateOp {
    Add,
    Sub,
    Xor,
    /// Guarded: traps unless the factor is nonzero and nothing overflows.
    Mul,
    /// Guarded: traps unless the factor is nonzero and the division is exact.
    Div,
}

impl UpdateOp {
    pub fn inverse(self) -> Self {
        match self {
            Self::Add => Self::Sub,
            Self::Sub => Self::Add,
            Self::Xor => Self::Xor,
            Self::Mul => Self::Div,
            Self::Div => Self::Mul,
        }
    }
}

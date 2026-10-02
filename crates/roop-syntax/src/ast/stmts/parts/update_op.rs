#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateOp {
    Add,
    Sub,
    Xor,
}

impl UpdateOp {
    pub fn inverse(self) -> Self {
        match self {
            Self::Add => Self::Sub,
            Self::Sub => Self::Add,
            Self::Xor => Self::Xor,
        }
    }
}

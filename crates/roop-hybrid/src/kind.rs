use std::fmt;

/// What a name was declared as: `qubit`, `bit` or `int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Qubit,
    Bit,
    Int,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let word = match self {
            Self::Qubit => "a qubit",
            Self::Bit => "a bit",
            Self::Int => "an int",
        };
        write!(f, "{word}")
    }
}

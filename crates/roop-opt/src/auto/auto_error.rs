use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum AutoError {
    StartsNonZero { name: String },
}

impl fmt::Display for AutoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StartsNonZero { name } => write!(
                f,
                "`auto ancilla {name}` must start at zero, since what is kept comes back as zero"
            ),
        }
    }
}

impl std::error::Error for AutoError {}

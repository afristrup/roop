use std::fmt;

/// Why a function could not be translated. The rest of the program still is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeanError {
    Unsupported(String),
    Unknown(String),
}

impl fmt::Display for LeanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unsupported(what) => write!(f, "not modelled yet: {what}"),
            Self::Unknown(what) => write!(f, "unknown {what}"),
        }
    }
}

impl std::error::Error for LeanError {}

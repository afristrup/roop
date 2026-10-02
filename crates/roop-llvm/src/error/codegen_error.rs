use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum CodegenError {
    UnknownName { kind: &'static str, name: String },
    TypeMismatch { expected: String, found: String },
    InvalidOperand(&'static str),
    Unsupported(&'static str),
    Uninstantiated(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownName { kind, name } => write!(f, "unknown {kind} `{name}`"),
            Self::TypeMismatch { expected, found } => {
                write!(f, "expected {expected}, found {found}")
            }
            Self::InvalidOperand(what) => write!(f, "invalid operand: {what}"),
            Self::Uninstantiated(len) => {
                write!(f, "generic length `{len}` was not instantiated")
            }
            Self::Unsupported(what) => write!(f, "unsupported in code generation: {what}"),
        }
    }
}

impl std::error::Error for CodegenError {}

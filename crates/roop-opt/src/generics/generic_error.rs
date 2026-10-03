use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum GenericError {
    NotGeneric(String),
    Arity {
        callee: String,
        expected: usize,
        found: usize,
    },
    NotConstant(String),
    Unbound(String),
    Negative(String),
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotGeneric(name) => write!(f, "`{name}` is not generic but was given lengths"),
            Self::Arity {
                callee,
                expected,
                found,
            } => write!(f, "`{callee}` takes {expected} lengths, found {found}"),
            Self::NotConstant(callee) => {
                write!(
                    f,
                    "lengths for `{callee}` must be constants or generic parameters"
                )
            }
            Self::Unbound(len) => write!(f, "length `{len}` is not a parameter of this function"),
            Self::Negative(callee) => write!(f, "`{callee}` was given a negative length"),
        }
    }
}

impl std::error::Error for GenericError {}

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum EinsumError {
    Empty(String),
    BadCharacter { name: String, found: char },
    MissingLabel { name: String, label: char },
    RepeatedOutput { name: String, label: char },
    BadType(String),
}

impl fmt::Display for EinsumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty(name) => write!(f, "`{name}`: the subscripts name no operand"),
            Self::BadCharacter { name, found } => write!(
                f,
                "`{name}`: `{found}` in the subscripts, which are letters, commas and `->`"
            ),
            Self::MissingLabel { name, label } => {
                write!(f, "`{name}`: the output has `{label}`, which no input has")
            }
            Self::RepeatedOutput { name, label } => write!(
                f,
                "`{name}`: `{label}` is twice in the output, which would write one place twice"
            ),
            Self::BadType(name) => write!(f, "`{name}`: the numbers must be i64 or f64"),
        }
    }
}

impl std::error::Error for EinsumError {}

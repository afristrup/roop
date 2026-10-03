use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum BennettError {
    UnknownTarget { name: String, target: String },
    NotReversible { name: String, target: String },
    Chained { name: String, target: String },
    Uncopyable { name: String, param: String },
    NameTaken { name: String, param: String },
}

impl fmt::Display for BennettError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownTarget { name, target } => {
                write!(
                    f,
                    "`{name}` is the Bennett version of `{target}`, which is not a function"
                )
            }
            Self::NotReversible { name, target } => {
                write!(
                    f,
                    "`{name}`: `{target}` is irreversible and has no inverse to uncompute with"
                )
            }
            Self::Chained { name, target } => {
                write!(f, "`{name}`: `{target}` is itself a Bennett version")
            }
            Self::Uncopyable { name, param } => write!(
                f,
                "`{name}`: the result `{param}` cannot be copied, only numbers, bools, structs and arrays of them can"
            ),
            Self::NameTaken { name, param } => {
                write!(f, "`{name}`: the parameter `{param}_out` already exists")
            }
        }
    }
}

impl std::error::Error for BennettError {}

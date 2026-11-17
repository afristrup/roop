use crate::Kind;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum HybridError {
    Lex {
        line: usize,
        found: char,
    },
    Number {
        line: usize,
        text: String,
    },
    Parse {
        line: usize,
        expected: String,
        found: String,
    },
    Undeclared {
        line: usize,
        name: String,
    },
    Redeclared {
        line: usize,
        name: String,
    },
    Kind {
        line: usize,
        name: String,
        expected: Kind,
        found: Kind,
    },
    Type {
        line: usize,
        expected: Kind,
    },
    Arity {
        line: usize,
        gate: String,
        expected: usize,
        found: usize,
    },
    SharedQubit {
        line: usize,
        qubit: String,
    },
    Rotation {
        line: usize,
        k: i64,
    },
    Branches {
        line: usize,
        names: Vec<String>,
    },
    LoopAllocation {
        line: usize,
        name: String,
    },
    Overflow,
    NegativeExponent(i64),
    TooManyQubits {
        qubits: usize,
        limit: usize,
    },
    TooManyStores(usize),
}

impl fmt::Display for HybridError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lex { line, found } => write!(f, "line {line}: unexpected `{found}`"),
            Self::Number { line, text } => {
                write!(f, "line {line}: `{text}` does not fit in 64 bits")
            }
            Self::Parse {
                line,
                expected,
                found,
            } => {
                write!(f, "line {line}: expected {expected}, found {found}")
            }
            Self::Undeclared { line, name } => write!(f, "line {line}: `{name}` is not declared"),
            Self::Redeclared { line, name } => {
                write!(f, "line {line}: `{name}` is already declared")
            }
            Self::Kind {
                line,
                name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "line {line}: `{name}` must be {expected}, and it is {found}"
                )
            }
            Self::Type { line, expected } => {
                write!(f, "line {line}: this expression must be {expected}")
            }
            Self::Arity {
                line,
                gate,
                expected,
                found,
            } => {
                let unit = if *expected == 1 { "qubit" } else { "qubits" };
                write!(
                    f,
                    "line {line}: `{gate}` takes {expected} {unit}, not {found}"
                )
            }
            Self::SharedQubit { line, qubit } => {
                write!(
                    f,
                    "line {line}: `{qubit}` appears twice in one gate, which is not unitary"
                )
            }
            Self::Rotation { line, k } => {
                write!(
                    f,
                    "line {line}: R_{k} is not a rotation; k runs from 0 to 62"
                )
            }
            Self::Branches { line, names } => write!(
                f,
                "line {line}: the branches of `if` must declare the same names; only one declares {}",
                names.join(", ")
            ),
            Self::LoopAllocation { line, name } => {
                write!(
                    f,
                    "line {line}: `{name}` is declared in a loop, which cannot allocate"
                )
            }
            Self::Overflow => write!(f, "integer arithmetic overflowed 64 bits"),
            Self::NegativeExponent(e) => write!(f, "the exponent {e} is negative"),
            Self::TooManyQubits { qubits, limit } => {
                write!(
                    f,
                    "{qubits} qubits; the exact interpreter takes at most {limit}"
                )
            }
            Self::TooManyStores(n) => {
                write!(
                    f,
                    "more than {n} classical states; the program branches too much to run exactly"
                )
            }
        }
    }
}

impl std::error::Error for HybridError {}

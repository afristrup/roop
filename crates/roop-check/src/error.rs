use roop_syntax::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum CheckError {
    SelfReferentialUpdate { var: String, span: Span },
    AncillaNotRestored { name: String, span: Span },
    AncillaTouchedInControlFlow { name: String, span: Span },
    NonExhaustiveMatch { span: Span },
    BorrowedPlaceUsed { var: String, span: Span },
    BorrowIndexModified { var: String, span: Span },
    TryCannotFail { span: Span },
    UnpairedBuild { name: String, span: Span },
    UnbuildNotInverse { name: String, span: Span },
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SelfReferentialUpdate { var, span } => write!(
                f,
                "state-destructive update: `{var}` appears on its own right-hand side at {}..{}",
                span.start, span.end
            ),
            Self::AncillaNotRestored { name, span } => write!(
                f,
                "ancilla `{name}` is not provably restored at block exit at {}..{}",
                span.start, span.end
            ),
            Self::AncillaTouchedInControlFlow { name, span } => write!(
                f,
                "ancilla `{name}` is modified inside control flow at {}..{}",
                span.start, span.end
            ),
            Self::BorrowedPlaceUsed { var, span } => write!(
                f,
                "`{var}` is used while borrowed at {}..{}",
                span.start, span.end
            ),
            Self::BorrowIndexModified { var, span } => write!(
                f,
                "`{var}` selects a borrowed place and is modified in the borrow at {}..{}",
                span.start, span.end
            ),
            Self::TryCannotFail { span } => write!(
                f,
                "try body has no exit assertion or call that could roll back at {}..{}",
                span.start, span.end
            ),
            Self::UnpairedBuild { name, span } => write!(
                f,
                "struct `{name}` must define both build and unbuild at {}..{}",
                span.start, span.end
            ),
            Self::UnbuildNotInverse { name, span } => write!(
                f,
                "unbuild of `{name}` is not provably the inverse of build at {}..{}",
                span.start, span.end
            ),
            Self::NonExhaustiveMatch { span } => write!(
                f,
                "match has no wildcard arm at {}..{}",
                span.start, span.end
            ),
        }
    }
}

impl std::error::Error for CheckError {}

use super::{CheckError, check_block};
use roop_syntax::{MatchArm, Pattern, Span};

/// A reversible match must be total, so it needs a wildcard arm.
pub fn check_match(arms: &[MatchArm], span: Span) -> Result<(), CheckError> {
    if !arms.iter().any(|arm| arm.pattern == Pattern::Wildcard) {
        return Err(CheckError::NonExhaustiveMatch { span });
    }
    arms.iter().try_for_each(|arm| check_block(&arm.body))
}

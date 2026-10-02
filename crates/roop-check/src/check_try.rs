use super::{CheckError, can_roll_back, check_block};
use roop_syntax::{Block, Span};

/// A `try` whose body cannot fail has a dead handler, which is a mistake.
pub fn check_try(body: &Block, handler: &Block, span: Span) -> Result<(), CheckError> {
    if !can_roll_back(body) {
        return Err(CheckError::TryCannotFail { span });
    }
    check_block(body)?;
    check_block(handler)
}

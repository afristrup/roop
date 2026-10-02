use crate::{
    CheckError, Scope, body_effects, can_roll_back, check_block, find_non_unwindable, place_root,
};
use roop_syntax::{Block, Place, Span};

/// A `try` whose body cannot fail has a dead handler, which is a mistake. With
/// an outcome the statement is reversible, which is only true when a failure
/// can be undone by running the finished statements backward, and when the
/// outcome is left alone by the body and the handler: it is zero going in and
/// set going out, so neither can read it either.
pub fn check_try(
    body: &Block,
    handler: &Block,
    outcome: Option<&Place>,
    span: Span,
    scope: Scope,
) -> Result<(), CheckError> {
    if let Some(outcome) = outcome {
        for block in [body, handler] {
            if let Some((what, at)) =
                find_non_unwindable(block, scope.logged, scope.non_atomic_fns)
            {
                return Err(CheckError::NotUnwindable { what, span: at });
            }
            let effects = body_effects(block);
            let root = place_root(outcome);
            if effects.writes.contains(root) || effects.reads.contains(root) {
                return Err(CheckError::TryOutcomeWritten {
                    var: place_root(outcome).into(),
                    span,
                });
            }
        }
    }
    if !can_roll_back(body) {
        return Err(CheckError::TryCannotFail { span });
    }
    check_block(body, scope)?;
    check_block(handler, scope)
}

use crate::{Ctx, Env, LeanError, place_type};
use roop_syntax::{Place, Type};

/// The element type of the stack at `place`.
pub fn stack_elem(cx: &Ctx, env: &Env, place: &Place) -> Result<Type, LeanError> {
    match place_type(cx, env, place)? {
        Type::Stack(elem, _) => Ok(*elem),
        _ => Err(LeanError::Unsupported("a stack place is required".into())),
    }
}

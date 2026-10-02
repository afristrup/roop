use crate::{
    Ctx, Env, LeanError, Out, assign_place, lean_zero, place_type, read_place, stack_elem,
};
use roop_syntax::Place;

/// Moves the value at `place` onto the stack and leaves zero behind.
pub fn move_onto(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    stack: &Place,
    place: &Place,
) -> Result<(), LeanError> {
    stack_elem(cx, env, stack)?;
    let zero = lean_zero(&place_type(cx, env, place)?)?;
    let pushed = format!(
        "(\u{2190} Roop.Stack.push {} {})",
        read_place(cx, env, stack)?,
        read_place(cx, env, place)?
    );
    assign_place(cx, env, out, stack, &pushed)?;
    assign_place(cx, env, out, place, &zero)
}

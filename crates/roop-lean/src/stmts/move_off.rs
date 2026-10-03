use crate::{
    Ctx, Env, LeanError, Out, assign_place, lean_zero, place_type, read_place, stack_elem,
};
use roop_syntax::Place;

/// Moves the top of the stack into `place`, which must be zero.
pub fn move_off(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    stack: &Place,
    place: &Place,
) -> Result<(), LeanError> {
    stack_elem(cx, env, stack)?;
    let zero = lean_zero(&place_type(cx, env, place)?)?;
    out.line(&format!(
        "Roop.check ({} == {zero}) Roop.Fail.assertion",
        read_place(cx, env, place)?
    ));
    let popped = out.fresh("__pop");
    out.line(&format!(
        "let {popped} \u{2190} Roop.Stack.pop {}",
        read_place(cx, env, stack)?
    ));
    assign_place(cx, env, out, stack, &format!("{popped}.2"))?;
    assign_place(cx, env, out, place, &format!("{popped}.1"))
}

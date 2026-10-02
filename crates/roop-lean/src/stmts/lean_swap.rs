use crate::{Ctx, Env, LeanError, Out, assign_place, read_place};
use roop_syntax::Place;

pub fn lean_swap(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    a: &Place,
    b: &Place,
) -> Result<(), LeanError> {
    let (ta, tb) = (out.fresh("__swap"), out.fresh("__swap"));
    out.line(&format!("let {ta} := {}", read_place(cx, env, a)?));
    out.line(&format!("let {tb} := {}", read_place(cx, env, b)?));
    assign_place(cx, env, out, a, &tb)?;
    assign_place(cx, env, out, b, &ta)
}

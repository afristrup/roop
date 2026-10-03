use crate::{Ctx, Dir, Env, LeanError, Out, move_off, move_onto};
use roop_syntax::Place;

/// `pop s -> x` moves the top into `x`; backward it is a `push` of `x`.
pub fn lean_pop(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    stack: &Place,
    target: &Place,
    dir: Dir,
) -> Result<(), LeanError> {
    match dir {
        Dir::Forward => move_off(cx, env, out, stack, target),
        Dir::Backward => move_onto(cx, env, out, stack, target),
    }
}

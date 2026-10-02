use crate::{Ctx, Dir, Env, LeanError, Out, move_off, move_onto};
use roop_syntax::Place;

/// `push s <- x` moves `x` onto the stack; backward it is a `pop` into `x`.
pub fn lean_push(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    stack: &Place,
    source: &Place,
    dir: Dir,
) -> Result<(), LeanError> {
    match dir {
        Dir::Forward => move_onto(cx, env, out, stack, source),
        Dir::Backward => move_off(cx, env, out, stack, source),
    }
}

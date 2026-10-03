use crate::{Ctx, Dir, Env, LeanError, Out, lean_block};
use roop_syntax::{Block, Place};

/// Destroying updates inside push what they destroy on `history`.
pub fn lean_logged(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    history: &Place,
    body: &Block,
    dir: Dir,
) -> Result<(), LeanError> {
    env.logged.push(history.clone());
    let result = lean_block(cx, env, out, body, dir);
    env.logged.pop();
    result
}

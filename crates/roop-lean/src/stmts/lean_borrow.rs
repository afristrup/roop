use crate::{Ctx, Dir, Env, LeanError, Out, assign_place, esc, lean_block, place_type, read_place};
use roop_syntax::{Block, Place};

/// A borrow is a forward/backward function pair in Aeneas: read the place into
/// a local, work on the local, write it back when the borrow ends.
pub fn lean_borrow(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    name: &str,
    source: &Place,
    body: &Block,
    dir: Dir,
) -> Result<(), LeanError> {
    let ty = place_type(cx, env, source)?;
    out.line(&format!(
        "let mut {} := {}",
        esc(name),
        read_place(cx, env, source)?
    ));
    env.vars.push((name.into(), ty));
    let result = lean_block(cx, env, out, body, dir);
    env.vars.pop();
    result?;
    assign_place(cx, env, out, source, &esc(name))
}

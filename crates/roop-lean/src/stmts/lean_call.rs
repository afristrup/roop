use crate::{
    Ctx, Dir, Env, LeanError, Out, assign_place, esc, lean_expr, read_place, tuple_proj,
};
use roop_syntax::{Expr, Type};

/// Calls return the callee's mutable parameters; they are written back into the
/// places passed in. `uncall`, or any call translated backward, runs `f_inv`.
pub fn lean_call(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    callee: &str,
    args: &[Expr],
    is_uncall: bool,
    dir: Dir,
) -> Result<(), LeanError> {
    let def = cx
        .fns
        .get(callee)
        .ok_or_else(|| LeanError::Unknown(format!("function `{callee}`")))?;
    if !cx.translated.contains(callee) {
        return Err(LeanError::Unknown(format!("function `{callee}`")));
    }
    let inverse = is_uncall != (dir == Dir::Backward);
    if inverse && def.irreversible {
        return Err(LeanError::Unsupported("the inverse of an irreversible function".into()));
    }
    let name = if inverse { format!("{callee}_inv") } else { callee.to_string() };
    let mut passed = Vec::new();
    let mut writeback = Vec::new();
    for (param, arg) in def.params.iter().zip(args) {
        match (&param.ty, arg) {
            (Type::Ref { mutable: true, .. }, Expr::Place(place)) => {
                passed.push(read_place(cx, env, place)?);
                writeback.push(place);
            }
            (Type::Ref { mutable: true, .. }, _) => {
                return Err(LeanError::Unsupported("a mutable argument that is not a place".into()));
            }
            _ => passed.push(lean_expr(cx, env, arg)?),
        }
    }
    let call = format!("{} {}", esc(&name), passed.join(" "));
    match writeback.len() {
        0 => out.line(call.trim_end()),
        n => {
            let result = out.fresh("__ret");
            out.line(&format!("let {result} \u{2190} {call}"));
            for (k, place) in writeback.into_iter().enumerate() {
                assign_place(cx, env, out, place, &tuple_proj(&result, k, n))?;
            }
        }
    }
    Ok(())
}

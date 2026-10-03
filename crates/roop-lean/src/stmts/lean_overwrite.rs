use crate::{
    Ctx, Dir, Env, LeanError, Out, assign_place, is_float, lean_expr_as, overwrite_value,
    place_type, read_place, stack_elem,
};
use roop_syntax::{Expr, OverwriteOp, Place};

/// Destroys the old value. Outside `logged` it only exists going forward.
/// Inside, the old value goes on the history stack, and backward the update
/// checks that the target still holds its result, pops the old value and puts
/// it back.
pub fn lean_overwrite(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    target: &Place,
    op: OverwriteOp,
    value: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    let ty = place_type(cx, env, target)?;
    let float = is_float(&ty);
    let (old, e) = (
        read_place(cx, env, target)?,
        lean_expr_as(cx, env, value, &ty)?,
    );
    let Some(history) = env.logged.last() else {
        if dir == Dir::Backward {
            return Err(LeanError::Unsupported("an overwrite run backward".into()));
        }
        return assign_place(cx, env, out, target, &overwrite_value(op, float, &old, &e)?);
    };
    if stack_elem(cx, env, history)? != place_type(cx, env, target)? {
        return Err(LeanError::Unsupported(
            "a history of another type than the value it logs".into(),
        ));
    }
    let stack = read_place(cx, env, history)?;
    match dir {
        Dir::Forward => {
            let pushed = format!("(\u{2190} Roop.Stack.push {stack} {old})");
            assign_place(cx, env, out, history, &pushed)?;
            assign_place(cx, env, out, target, &overwrite_value(op, float, &old, &e)?)
        }
        Dir::Backward => {
            let popped = out.fresh("__pop");
            out.line(&format!("let {popped} \u{2190} Roop.Stack.pop {stack}"));
            let expected = overwrite_value(op, float, &format!("{popped}.1"), &e)?;
            out.line(&format!(
                "Roop.check ({old} == {expected}) Roop.Fail.assertion"
            ));
            assign_place(cx, env, out, history, &format!("{popped}.2"))?;
            assign_place(cx, env, out, target, &format!("{popped}.1"))
        }
    }
}

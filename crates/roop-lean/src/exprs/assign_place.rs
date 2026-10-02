use crate::{Ctx, Env, LeanError, Out, esc, lean_expr, read_place};
use roop_syntax::Place;

/// Writes `value` into a place by rebuilding the enclosing struct or array
/// and assigning that to the root variable.
pub fn assign_place(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    place: &Place,
    value: &str,
) -> Result<(), LeanError> {
    match place {
        Place::Var(name) => out.line(&format!("{} := {value}", esc(name))),
        Place::Field(base, field) => {
            let whole = format!(
                "{{ {} with {} := {value} }}",
                read_place(cx, env, base)?,
                esc(field)
            );
            assign_place(cx, env, out, base, &whole)?;
        }
        Place::Index(base, index) => {
            let whole = format!(
                "(\u{2190} Roop.aset {} {} {value})",
                read_place(cx, env, base)?,
                lean_expr(cx, env, index)?
            );
            assign_place(cx, env, out, base, &whole)?;
        }
    }
    Ok(())
}

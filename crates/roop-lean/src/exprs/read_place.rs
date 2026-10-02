use crate::{Ctx, Env, LeanError, esc, lean_expr};
use roop_syntax::Place;

/// The value stored at a place.
pub fn read_place(cx: &Ctx, env: &Env, place: &Place) -> Result<String, LeanError> {
    Ok(match place {
        Place::Var(name) => esc(name),
        Place::Field(base, field) => {
            format!("({}).{}", read_place(cx, env, base)?, esc(field))
        }
        Place::Index(base, index) => format!(
            "(\u{2190} Roop.aget {} {})",
            read_place(cx, env, base)?,
            lean_expr(cx, env, index)?
        ),
    })
}

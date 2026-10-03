use crate::{Ctx, Env, LeanError, esc, lean_expr, literal_index};
use roop_syntax::Place;

/// The value stored at a place. An access at a literal index reads with the
/// index as a natural number, so that simplification settles its bound at once.
pub fn read_place(cx: &Ctx, env: &Env, place: &Place) -> Result<String, LeanError> {
    Ok(match place {
        Place::Var(name) => esc(name),
        Place::Field(base, field) => {
            format!("({}).{}", read_place(cx, env, base)?, esc(field))
        }
        Place::Index(base, index) => match literal_index(index) {
            Some(k) => format!("(\u{2190} Roop.agetN {} {k})", read_place(cx, env, base)?),
            None => format!(
                "(\u{2190} Roop.aget {} {})",
                read_place(cx, env, base)?,
                lean_expr(cx, env, index)?
            ),
        },
    })
}

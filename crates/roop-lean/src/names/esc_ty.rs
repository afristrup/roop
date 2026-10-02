use crate::esc;

/// User structs and enums live in the `Ty` namespace.
pub fn esc_ty(name: &str) -> String {
    format!("Ty.{}", esc(name))
}

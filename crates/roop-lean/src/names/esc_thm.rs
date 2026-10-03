use crate::esc;

/// Theorems live in the `Thm` namespace, next to the functions they are about.
pub fn esc_thm(name: &str) -> String {
    format!("Thm.{}", esc(name))
}

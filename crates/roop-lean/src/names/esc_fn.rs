use crate::esc;

/// Functions and their inverses live in the `Fn` namespace, so a roop name
/// like `xor` cannot collide with something in Lean.
pub fn esc_fn(name: &str) -> String {
    format!("Fn.{}", esc(name))
}

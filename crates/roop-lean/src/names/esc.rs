/// A Lean identifier for a roop name. Guillemets keep reserved words legal.
pub fn esc(name: &str) -> String {
    format!("\u{ab}{name}\u{bb}")
}

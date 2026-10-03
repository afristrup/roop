use std::sync::OnceLock;

/// The program's arguments, set once from the C `main`.
pub static ARGS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();

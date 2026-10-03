use crate::LeanError;
use roop_syntax::Type;

/// `e as T` where Lean has the conversion: between `i64`, `u8` and `bool`.
pub fn lean_cast(inner: &str, from: &Type, to: &Type) -> Result<String, LeanError> {
    let name = |t: &Type| match t {
        Type::Named(n) => n.clone(),
        _ => String::new(),
    };
    Ok(match (name(from).as_str(), name(to).as_str()) {
        (a, b) if a == b => inner.to_string(),
        ("i64", "u8") => format!("(BitVec.setWidth 8 {inner})"),
        ("u8", "i64") => format!("(BitVec.setWidth 64 {inner})"),
        ("bool", "i64") => format!("(if {inner} then (1 : Roop.I64) else 0)"),
        ("bool", "u8") => format!("(if {inner} then (1 : Roop.U8) else 0)"),
        _ => {
            return Err(LeanError::Unsupported(
                "a conversion involving floats".into(),
            ));
        }
    })
}

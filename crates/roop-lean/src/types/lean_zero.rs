use crate::LeanError;
use roop_syntax::Type;

/// What a place holds after its value was moved out.
pub fn lean_zero(ty: &Type) -> Result<String, LeanError> {
    match ty {
        Type::Named(n) if n == "i64" => Ok("(0 : Roop.I64)".into()),
        Type::Named(n) if n == "bool" => Ok("false".into()),
        Type::Named(n) if n == "f64" => Ok("(0.0 : Float)".into()),
        _ => Err(LeanError::Unsupported(
            "a stack of values that are not numbers or booleans".into(),
        )),
    }
}

use roop_syntax::Type;

/// The length of a vector of doubles, `[f64; len]`.
pub fn vector_len(ty: &Type) -> Option<i64> {
    let Type::Array(cell, len) = ty else {
        return None;
    };
    if !matches!(&**cell, Type::Named(name) if name == "f64") {
        return None;
    }
    i64::try_from(*len).ok()
}

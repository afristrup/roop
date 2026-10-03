use roop_syntax::Type;

/// The rows and columns of a matrix of numbers of one type, `[[f64; cols]; rows]`.
pub fn matrix_dims(ty: &Type, element: &str) -> Option<(i64, i64)> {
    let Type::Array(row, rows) = ty else {
        return None;
    };
    let Type::Array(cell, cols) = &**row else {
        return None;
    };
    if !matches!(&**cell, Type::Named(name) if name == element) {
        return None;
    }
    Some((i64::try_from(*rows).ok()?, i64::try_from(*cols).ok()?))
}

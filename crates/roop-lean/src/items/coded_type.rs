use roop_syntax::Type;

/// Whether the prelude counts the values of the type: numbers, bools, and
/// arrays of them. A stack or a float has no finite count in the model.
pub fn coded_type(ty: &Type) -> bool {
    match ty {
        Type::Named(name) => name == "i64" || name == "bool",
        Type::Array(elem, _) => coded_type(elem),
        Type::Ref { inner, .. } => coded_type(inner),
        _ => false,
    }
}

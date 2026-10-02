use roop_syntax::Type;

/// Source-level type name recorded in Metal reflection metadata.
pub fn type_label(ty: &Type) -> String {
    match ty {
        Type::Named(n) => match n.as_str() {
            "i64" => "long".into(),
            "f64" => "double".into(),
            other => other.into(),
        },
        Type::Array(elem, len) => format!("{}[{len}]", type_label(elem)),
        Type::Ref { inner, .. } => type_label(inner),
    }
}

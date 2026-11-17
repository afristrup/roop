use roop_syntax::Type;

/// Whether a value of this type holds a qubit: a `qubit`, or an array of them.
pub fn holds_qubit(ty: &Type) -> bool {
    match ty {
        Type::Named(name) => name == "qubit",
        Type::Ref { inner, .. } => holds_qubit(inner),
        Type::Array(elem, _) | Type::Stack(elem, _) | Type::Param { elem, .. } => holds_qubit(elem),
    }
}

use roop_syntax::Type;

pub fn unref(ty: &Type) -> Type {
    match ty {
        Type::Ref { inner, .. } => (**inner).clone(),
        other => other.clone(),
    }
}

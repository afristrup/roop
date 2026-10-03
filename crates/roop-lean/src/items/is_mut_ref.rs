use roop_syntax::Type;

pub fn is_mut_ref(ty: &Type) -> bool {
    matches!(ty, Type::Ref { mutable: true, .. })
}

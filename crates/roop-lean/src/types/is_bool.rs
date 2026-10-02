use roop_syntax::Type;

pub fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Named(n) if n == "bool")
}

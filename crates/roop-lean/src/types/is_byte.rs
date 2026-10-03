use roop_syntax::Type;

pub fn is_byte(ty: &Type) -> bool {
    matches!(ty, Type::Named(n) if n == "u8")
}

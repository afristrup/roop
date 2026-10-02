use roop_syntax::Type;

pub fn is_float(ty: &Type) -> bool {
    matches!(ty, Type::Named(n) if n == "f64")
}

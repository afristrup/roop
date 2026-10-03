use crate::esc_ty;
use roop_syntax::Type;

/// The Lean type of a roop type. A reference stands for what it points at.
pub fn lean_type(ty: &Type) -> String {
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" => "Roop.I64".into(),
            "u8" => "Roop.U8".into(),
            "f64" => "Float".into(),
            "bool" => "Bool".into(),
            other => esc_ty(other),
        },
        Type::Array(elem, len) => format!("(Vector {} {len})", lean_type(elem)),
        Type::Stack(elem, cap) => format!("(Roop.Stack {} {cap})", lean_type(elem)),
        Type::Ref { inner, .. } => lean_type(inner),
        Type::Param { len, .. } => panic!("generic length `{len}` was not instantiated"),
    }
}

use crate::OnName;
use roop_syntax::Type;

pub fn visit_type(ty: &mut Type, on: OnName) {
    match ty {
        Type::Named(name) => on(name),
        Type::Ref { inner, .. } => visit_type(inner, on),
        Type::Array(inner, _) | Type::Stack(inner, _) => visit_type(inner, on),
    }
}

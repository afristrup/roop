use crate::{Type, Visitor};

pub fn walk_type(v: &mut dyn Visitor, ty: &mut Type) {
    v.ty(ty);
    match ty {
        Type::Named(name) => v.name(name),
        Type::Ref { inner, .. }
        | Type::Array(inner, _)
        | Type::Stack(inner, _)
        | Type::Param { elem: inner, .. } => walk_type(v, inner),
    }
}

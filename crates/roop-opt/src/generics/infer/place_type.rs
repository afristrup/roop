use crate::Scope;
use roop_syntax::{Place, Type};

fn without_ref(ty: &Type) -> &Type {
    match ty {
        Type::Ref { inner, .. } => without_ref(inner),
        other => other,
    }
}

/// The type of a place, when the scope and the structs say.
pub fn place_type(scope: &Scope, place: &Place) -> Option<Type> {
    match place {
        Place::Var(name) => scope.lookup(name).map(|t| without_ref(t).clone()),
        Place::Field(base, field) => {
            let Type::Named(name) = place_type(scope, base)? else {
                return None;
            };
            let def = scope.structs.get(name.as_str())?;
            def.fields
                .iter()
                .find(|f| f.name == *field)
                .map(|f| f.ty.clone())
        }
        Place::Index(base, _) => match place_type(scope, base)? {
            Type::Array(elem, _)
            | Type::Param {
                elem, stack: false, ..
            } => Some(*elem),
            _ => None,
        },
    }
}

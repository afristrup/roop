use crate::{Ctx, Env, LeanError};
use roop_syntax::{Place, Type};

pub fn place_type(cx: &Ctx, env: &Env, place: &Place) -> Result<Type, LeanError> {
    match place {
        Place::Var(name) => match env.lookup(name) {
            Some(Type::Ref { inner, .. }) => Ok((**inner).clone()),
            Some(ty) => Ok(ty.clone()),
            None => Err(LeanError::Unknown(format!("variable `{name}`"))),
        },
        Place::Field(base, field) => {
            let Type::Named(owner) = place_type(cx, env, base)? else {
                return Err(LeanError::Unsupported("field of a non-struct".into()));
            };
            let def = cx
                .structs
                .get(owner.as_str())
                .ok_or_else(|| LeanError::Unknown(format!("struct `{owner}`")))?;
            def.fields
                .iter()
                .find(|f| &f.name == field)
                .map(|f| f.ty.clone())
                .ok_or_else(|| LeanError::Unknown(format!("field `{field}`")))
        }
        Place::Index(base, _) => match place_type(cx, env, base)? {
            Type::Array(elem, _) => Ok(*elem),
            _ => Err(LeanError::Unsupported("indexing a non-array".into())),
        },
    }
}

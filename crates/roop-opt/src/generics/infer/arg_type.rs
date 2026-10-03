use crate::{Scope, place_type};
use roop_syntax::{Expr, Type};

/// The type of an argument, when it has one the inference can use: a place
/// with a known type, or a string literal.
pub fn arg_type(scope: &Scope, arg: &Expr) -> Option<Type> {
    match arg {
        Expr::Place(place) => place_type(scope, place),
        Expr::Str(bytes) => Some(Type::Array(
            Box::new(Type::Named("u8".into())),
            bytes.len() as u64,
        )),
        _ => None,
    }
}

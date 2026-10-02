use crate::Ctx;
use roop_syntax::Type;

/// Whether values of the type contain `f64`, which Apple GPUs lack.
pub fn has_f64(ctx: &Ctx, ty: &Type) -> bool {
    match ty {
        Type::Named(n) if n == "f64" => true,
        Type::Named(n) => ctx
            .structs
            .get(n.as_str())
            .is_some_and(|def| def.fields.iter().any(|f| has_f64(ctx, &f.ty))),
        Type::Array(elem, _) | Type::Stack(elem, _) | Type::Param { elem, .. } => {
            has_f64(ctx, elem)
        }
        Type::Ref { inner, .. } => has_f64(ctx, inner),
    }
}

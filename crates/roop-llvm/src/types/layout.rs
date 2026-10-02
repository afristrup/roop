use crate::{CodegenError, Ctx};
use roop_syntax::Type;

/// Size and alignment in bytes under LLVM's natural layout, which host and
/// both GPU data layouts agree on.
pub fn layout(ctx: &Ctx, ty: &Type) -> Result<(u64, u64), CodegenError> {
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" | "f64" => Ok((8, 8)),
            "bool" => Ok((1, 1)),
            _ if ctx.enums.contains_key(name.as_str()) => Ok((4, 4)),
            _ => {
                let def = ctx.structs.get(name.as_str()).ok_or_else(|| {
                    CodegenError::UnknownName {
                        kind: "type",
                        name: name.clone(),
                    }
                })?;
                let (mut offset, mut widest) = (0u64, 1u64);
                for field in &def.fields {
                    let (size, align) = layout(ctx, &field.ty)?;
                    offset = offset.next_multiple_of(align) + size;
                    widest = widest.max(align);
                }
                Ok((offset.next_multiple_of(widest), widest))
            }
        },
        Type::Array(elem, len) => {
            let (size, align) = layout(ctx, elem)?;
            Ok((size * len, align))
        }
        Type::Ref { .. } => Ok((8, 8)),
    }
}

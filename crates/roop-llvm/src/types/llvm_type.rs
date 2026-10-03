use crate::{CodegenError, Ctx};
use roop_syntax::Type;

pub fn llvm_type(ctx: &Ctx, ty: &Type) -> Result<String, CodegenError> {
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" => Ok("i64".into()),
            "u8" => Ok("i8".into()),
            "f64" => Ok("double".into()),
            "bool" => Ok("i1".into()),
            _ if ctx.structs.contains_key(name.as_str()) => Ok(format!("%{name}")),
            _ if ctx.enums.contains_key(name.as_str()) => Ok("i32".into()),
            _ => Err(CodegenError::UnknownName {
                kind: "type",
                name: name.clone(),
            }),
        },
        Type::Ref { .. } => Ok("ptr".into()),
        Type::Array(elem, len) => Ok(format!("[{len} x {}]", llvm_type(ctx, elem)?)),
        Type::Param { len, .. } => Err(CodegenError::Uninstantiated(len.clone())),
        Type::Stack(elem, cap) => Ok(format!("{{ i64, [{cap} x {}] }}", llvm_type(ctx, elem)?)),
    }
}

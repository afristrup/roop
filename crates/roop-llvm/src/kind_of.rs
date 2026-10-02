use super::{CodegenError, Ctx, Kind};
use roop_syntax::Type;

pub fn kind_of(ctx: &Ctx, ty: &Type) -> Result<Kind, CodegenError> {
    match ty {
        Type::Named(name) => match name.as_str() {
            "i64" => Ok(Kind::Int),
            "f64" => Ok(Kind::Float),
            "bool" => Ok(Kind::Bool),
            _ if ctx.enums.contains_key(name.as_str()) => Ok(Kind::Enum),
            _ => Err(CodegenError::InvalidOperand("operand must be a scalar")),
        },
        _ => Err(CodegenError::InvalidOperand("operand must be a scalar")),
    }
}

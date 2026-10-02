use super::{CodegenError, FnGen, Value, bool_type, llvm_type};
use roop_syntax::Pattern;

pub fn pattern_test(g: &mut FnGen, s: &Value, pattern: &Pattern) -> Result<Value, CodegenError> {
    let rhs = match pattern {
        Pattern::Wildcard => {
            return Ok(Value {
                reg: "true".into(),
                ty: bool_type(),
            });
        }
        Pattern::Int(i) => i.to_string(),
        Pattern::Bool(b) => b.to_string(),
        Pattern::Variant(enum_name, variant) => g
            .ctx
            .enums
            .get(enum_name.as_str())
            .and_then(|def| def.variants.iter().position(|v| v == variant))
            .map(|i| i.to_string())
            .ok_or_else(|| CodegenError::UnknownName {
                kind: "variant",
                name: format!("{enum_name}::{variant}"),
            })?,
    };
    let ty = llvm_type(g.ctx, &s.ty)?;
    let reg = format!("%{}", g.fresh("t"));
    g.emit(&format!("{reg} = icmp eq {ty} {}, {rhs}", s.reg));
    Ok(Value { reg, ty: bool_type() })
}

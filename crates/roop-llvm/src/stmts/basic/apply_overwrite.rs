use crate::{CodegenError, FnGen, Kind, Value, kind_of, llvm_type};
use roop_syntax::OverwriteOp;

/// The register holding `old op value`; plain assignment is just `value`.
pub fn apply_overwrite(
    g: &mut FnGen,
    op: OverwriteOp,
    old: &Value,
    value: &Value,
) -> Result<String, CodegenError> {
    if op == OverwriteOp::Assign {
        return Ok(value.reg.clone());
    }
    let kind = kind_of(g.ctx, &old.ty)?;
    let instr = match (kind, op) {
        (Kind::Int, OverwriteOp::Rem) => "srem",
        (Kind::Float, OverwriteOp::Rem) => "frem",
        _ => {
            return Err(CodegenError::InvalidOperand(
                "overwrite not defined for type",
            ));
        }
    };
    let ty = llvm_type(g.ctx, &old.ty)?;
    let new = format!("%{}", g.fresh("t"));
    g.emit(&format!("{new} = {instr} {ty} {}, {}", old.reg, value.reg));
    Ok(new)
}

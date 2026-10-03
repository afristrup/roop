use crate::{CodegenError, FnGen, gen_expr_as, gen_place, llvm_type, same_type, str_constant};
use roop_syntax::{Expr, Param, Type};

/// The LLVM arguments for a call: places go by address, read-only values are
/// spilled to a temporary, everything else is passed by value.
pub fn call_arguments(
    g: &mut FnGen,
    params: &[Param],
    args: &[Expr],
    external: bool,
) -> Result<Vec<String>, CodegenError> {
    if params.len() != args.len() {
        return Err(CodegenError::InvalidOperand("wrong number of arguments"));
    }
    let mut passed = Vec::new();
    for (param, arg) in params.iter().zip(args) {
        passed.push(match (&param.ty, arg) {
            (Type::Ref { inner, .. }, Expr::Place(place)) => {
                let slot = gen_place(g, place)?;
                if !external {
                    same_type(inner, &slot.ty)?;
                }
                format!("ptr {}", slot.addr)
            }
            (
                Type::Ref {
                    mutable: false,
                    inner,
                },
                Expr::Str(bytes),
            ) => {
                let found = Type::Array(Box::new(Type::Named("u8".into())), bytes.len() as u64);
                if !external {
                    same_type(inner, &found)?;
                }
                format!("ptr {}", str_constant(g, bytes))
            }
            (
                Type::Ref {
                    mutable: false,
                    inner,
                },
                value,
            ) => {
                let v = gen_expr_as(g, value, inner)?;
                same_type(inner, &v.ty)?;
                let ty = llvm_type(g.ctx, &v.ty)?;
                let addr = g.alloca(&ty);
                g.emit(&format!("store {ty} {}, ptr {addr}", v.reg));
                format!("ptr {addr}")
            }
            (Type::Ref { .. }, _) => {
                return Err(CodegenError::InvalidOperand(
                    "mutable reference argument must be a place",
                ));
            }
            (ty, value) => {
                let v = gen_expr_as(g, value, ty)?;
                same_type(ty, &v.ty)?;
                format!("{} {}", llvm_type(g.ctx, ty)?, v.reg)
            }
        });
    }
    Ok(passed)
}

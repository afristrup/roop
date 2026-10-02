use crate::{CodegenError, Dialect, Dir, FnGen, gen_expr, gen_place, llvm_type, same_type};
use roop_syntax::{Expr, Type};

/// `call` runs `@f` forward and `uncall` runs `@f_inv`; emitting backward
/// swaps the two.
pub fn gen_call(
    g: &mut FnGen,
    callee: &str,
    args: &[Expr],
    is_uncall: bool,
    dir: Dir,
) -> Result<(), CodegenError> {
    if g.dialect != Dialect::Host {
        return Err(CodegenError::Unsupported("call inside a GPU kernel"));
    }
    let def = g
        .ctx
        .fns
        .get(callee)
        .copied()
        .ok_or_else(|| CodegenError::UnknownName {
            kind: "function",
            name: callee.into(),
        })?;
    if def.params.len() != args.len() {
        return Err(CodegenError::InvalidOperand("wrong number of arguments"));
    }
    let mut passed = Vec::new();
    for (param, arg) in def.params.iter().zip(args) {
        passed.push(match (&param.ty, arg) {
            (Type::Ref { inner, .. }, Expr::Place(place)) => {
                let slot = gen_place(g, place)?;
                same_type(inner, &slot.ty)?;
                format!("ptr {}", slot.addr)
            }
            (
                Type::Ref {
                    mutable: false,
                    inner,
                },
                value,
            ) => {
                let v = gen_expr(g, value)?;
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
                let v = gen_expr(g, value)?;
                same_type(ty, &v.ty)?;
                format!("{} {}", llvm_type(g.ctx, ty)?, v.reg)
            }
        });
    }
    let inverse = is_uncall != (dir == Dir::Backward);
    if inverse && g.ctx.irreversible.contains(callee) {
        return Err(CodegenError::Unsupported(
            "running an irreversible function backward",
        ));
    }
    let symbol = if inverse {
        format!("{callee}_inv")
    } else {
        callee.into()
    };
    g.emit(&format!("call void @{symbol}({})", passed.join(", ")));
    Ok(())
}

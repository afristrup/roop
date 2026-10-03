use crate::{CodegenError, Dialect, Dir, FnGen, call_arguments, entry_symbol};
use roop_syntax::{Expr, FnDef};

/// `call` runs `@f` forward and `uncall` runs `@f_inv`; emitting backward
/// swaps the two.
pub fn gen_call(
    g: &mut FnGen,
    callee: &str,
    args: &[Expr],
    is_uncall: bool,
    dir: Dir,
) -> Result<(), CodegenError> {
    let (def, inverse) = resolve(g, callee, is_uncall, dir)?;
    let passed = call_arguments(g, &def.params, args, def.external)?;
    let base = entry_symbol(callee);
    let symbol = if inverse { format!("{base}_inv") } else { base };
    g.emit(&format!("call void @{symbol}({})", passed.join(", ")));
    Ok(())
}

/// The callee, and whether the call runs its inverse.
pub fn resolve<'a>(
    g: &FnGen<'a>,
    callee: &str,
    is_uncall: bool,
    dir: Dir,
) -> Result<(&'a FnDef, bool), CodegenError> {
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
    let inverse = is_uncall != (dir == Dir::Backward);
    if inverse && g.ctx.irreversible.contains(callee) {
        return Err(CodegenError::Unsupported(
            "running an irreversible function backward",
        ));
    }
    Ok((def, inverse))
}

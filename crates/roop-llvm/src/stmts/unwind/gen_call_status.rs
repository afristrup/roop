use crate::{CodegenError, Dir, FnGen, call_arguments, entry_symbol, resolve};
use roop_syntax::Expr;

/// A call to the failure-atomic variant of the callee, which returns nonzero
/// after undoing itself when it fails.
pub fn gen_call_status(
    g: &mut FnGen,
    callee: &str,
    args: &[Expr],
    is_uncall: bool,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let (def, inverse) = resolve(g, callee, is_uncall, dir)?;
    let passed = call_arguments(g, &def.params, args, def.external)?;
    if def.external {
        // The runtime reports a failure in a status argument, not by failing.
        let base = entry_symbol(callee);
        let symbol = if inverse { format!("{base}_inv") } else { base };
        g.emit(&format!("call void @{symbol}({})", passed.join(", ")));
        return Ok(());
    }
    let symbol = format!(
        "{}{}_try",
        entry_symbol(callee),
        if inverse { "_inv" } else { "" }
    );
    let status = format!("%{}", g.fresh("t"));
    g.emit(&format!(
        "{status} = call i32 @{symbol}({})",
        passed.join(", ")
    ));
    let ok = format!("%{}", g.fresh("t"));
    g.emit(&format!("{ok} = icmp eq i32 {status}, 0"));
    let go_on = g.fresh("L");
    g.emit(&format!("br i1 {ok}, label %{go_on}, label %{fail}"));
    g.label(&go_on);
    Ok(())
}

use crate::{
    AbortMode, CodegenError, Dir, FnGen, bool_type, gen_block, gen_expr, gen_unwinding, same_type,
};
use roop_syntax::{Block, Expr};

/// The exit assertion is checked at the end of each branch, where the branch
/// that ran is known, so a failure can undo exactly that branch.
pub fn unwind_if(
    g: &mut FnGen,
    cond: &Expr,
    then_block: &Block,
    else_block: &Block,
    exit: &Expr,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let (entry, assertion) = match dir {
        Dir::Forward => (cond, exit),
        Dir::Backward => (exit, cond),
    };
    let c = gen_expr(g, entry)?;
    same_type(&bool_type(), &c.ty)?;
    let (then_l, else_l, join) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
    g.emit(&format!(
        "br i1 {}, label %{then_l}, label %{else_l}",
        c.reg
    ));
    for (label, block) in [(then_l, then_block), (else_l, else_block)] {
        g.label(&label);
        gen_unwinding(g, block, dir, fail)?;
        let undo = g.fresh("L");
        let outer = std::mem::replace(&mut g.abort, AbortMode::Label(undo.clone()));
        let a = gen_expr(g, assertion)?;
        same_type(&bool_type(), &a.ty)?;
        g.emit(&format!("br i1 {}, label %{join}, label %{undo}", a.reg));
        g.abort = AbortMode::Trap;
        g.label(&undo);
        gen_block(g, block, dir.flip())?;
        g.emit(&format!("br label %{fail}"));
        g.abort = outer;
    }
    g.label(&join);
    Ok(())
}

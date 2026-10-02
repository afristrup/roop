use super::{CodegenError, Dir, FnGen, bool_type, gen_assert, gen_block, gen_expr, same_type};
use roop_syntax::{Block, Expr};

/// Backward, the exit assertion selects the branch and the entry condition
/// becomes the assertion.
pub fn gen_if(
    g: &mut FnGen,
    cond: &Expr,
    then_block: &Block,
    else_block: &Block,
    exit: &Expr,
    dir: Dir,
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
    g.label(&then_l);
    gen_block(g, then_block, dir)?;
    g.emit(&format!("br label %{join}"));
    g.label(&else_l);
    gen_block(g, else_block, dir)?;
    g.emit(&format!("br label %{join}"));
    g.label(&join);
    let a = gen_expr(g, assertion)?;
    gen_assert(g, &a)
}

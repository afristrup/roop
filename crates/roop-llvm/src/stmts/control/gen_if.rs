use crate::{CodegenError, Dir, FnGen, bool_type, gen_assert, gen_block, gen_expr, same_type};
use roop_syntax::{Block, Expr, UnOp};

/// The exit assertion holds after the then branch and fails after the else
/// branch, which is what lets it select the branch backward: there the exit
/// assertion is the entry condition and the entry condition is the assertion.
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
    let negated = Expr::Unary(UnOp::Not, Box::new(assertion.clone()));
    for (label, block, expected) in [
        (then_l, then_block, assertion),
        (else_l, else_block, &negated),
    ] {
        g.label(&label);
        gen_block(g, block, dir)?;
        let a = gen_expr(g, expected)?;
        gen_assert(g, &a)?;
        g.emit(&format!("br label %{join}"));
    }
    g.label(&join);
    Ok(())
}

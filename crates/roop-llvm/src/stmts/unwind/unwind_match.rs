use crate::{
    AbortMode, CodegenError, Dir, FnGen, bool_type, gen_block, gen_expr, gen_unwinding,
    pattern_test, same_type,
};
use roop_syntax::{Expr, MatchArm};

/// Like `unwind_if`, per arm: the arm's exit check sits at the end of the arm,
/// so a failure undoes that arm and nothing else.
pub fn unwind_match(
    g: &mut FnGen,
    scrutinee: &Expr,
    arms: &[MatchArm],
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let join = g.fresh("L");
    for arm in arms {
        let (arm_l, next_l) = (g.fresh("L"), g.fresh("L"));
        let select = match dir {
            Dir::Forward => {
                let s = gen_expr(g, scrutinee)?;
                pattern_test(g, &s, &arm.pattern)?
            }
            Dir::Backward => gen_expr(g, &arm.exit)?,
        };
        same_type(&bool_type(), &select.ty)?;
        g.emit(&format!(
            "br i1 {}, label %{arm_l}, label %{next_l}",
            select.reg
        ));
        g.label(&arm_l);
        gen_unwinding(g, &arm.body, dir, fail)?;
        let undo = g.fresh("L");
        let outer = std::mem::replace(&mut g.abort, AbortMode::Label(undo.clone()));
        let assertion = match dir {
            Dir::Forward => gen_expr(g, &arm.exit)?,
            Dir::Backward => {
                let s = gen_expr(g, scrutinee)?;
                pattern_test(g, &s, &arm.pattern)?
            }
        };
        same_type(&bool_type(), &assertion.ty)?;
        g.emit(&format!(
            "br i1 {}, label %{join}, label %{undo}",
            assertion.reg
        ));
        g.abort = AbortMode::Trap;
        g.label(&undo);
        gen_block(g, &arm.body, dir.flip())?;
        g.emit(&format!("br label %{fail}"));
        g.abort = outer;
        g.label(&next_l);
    }
    g.emit("call void @llvm.trap()");
    g.emit("unreachable");
    g.label(&join);
    Ok(())
}

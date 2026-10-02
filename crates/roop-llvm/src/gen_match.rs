use super::{CodegenError, Dir, FnGen, gen_assert, gen_block, gen_expr, pattern_test, same_type, bool_type};
use roop_syntax::{Expr, MatchArm};

/// Forward, patterns select the arm and the exit assertion is checked.
/// Backward, the exit assertions select the arm and the pattern is checked.
pub fn gen_match(
    g: &mut FnGen,
    scrutinee: &Expr,
    arms: &[MatchArm],
    dir: Dir,
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
        g.emit(&format!("br i1 {}, label %{arm_l}, label %{next_l}", select.reg));
        g.label(&arm_l);
        gen_block(g, &arm.body, dir)?;
        let assertion = match dir {
            Dir::Forward => gen_expr(g, &arm.exit)?,
            Dir::Backward => {
                let s = gen_expr(g, scrutinee)?;
                pattern_test(g, &s, &arm.pattern)?
            }
        };
        gen_assert(g, &assertion)?;
        g.emit(&format!("br label %{join}"));
        g.label(&next_l);
    }
    g.emit("call void @llvm.trap()");
    g.emit("unreachable");
    g.label(&join);
    Ok(())
}

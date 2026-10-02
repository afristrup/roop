use super::{
    CodegenError, Dir, FnGen, Value, bool_type, gen_assert, gen_block, gen_expr, same_type,
};
use roop_syntax::{Block, Expr};

/// Janus loop. The run is S1 (S2 S1)*, so its reverse is S1' (S2' S1')*: the
/// same shape with entry and until swapped and both blocks emitted backward.
pub fn gen_from(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), CodegenError> {
    let (start, first, second, stop) = match dir {
        Dir::Forward => (entry, body, step, until),
        Dir::Backward => (until, body, step, entry),
    };
    let on_entry = gen_expr(g, start)?;
    gen_assert(g, &on_entry)?;
    let (head, back, exit) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
    g.emit(&format!("br label %{head}"));
    g.label(&head);
    gen_block(g, first, dir)?;
    let done = gen_expr(g, stop)?;
    same_type(&bool_type(), &done.ty)?;
    g.emit(&format!("br i1 {}, label %{exit}, label %{back}", done.reg));
    g.label(&back);
    gen_block(g, second, dir)?;
    let again = gen_expr(g, start)?;
    let not_again = format!("%{}", g.fresh("t"));
    g.emit(&format!("{not_again} = xor i1 {}, true", again.reg));
    gen_assert(
        g,
        &Value {
            reg: not_again,
            ty: bool_type(),
        },
    )?;
    g.emit(&format!("br label %{head}"));
    g.label(&exit);
    Ok(())
}

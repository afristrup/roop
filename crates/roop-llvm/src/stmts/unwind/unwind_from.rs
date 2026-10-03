use crate::{
    AbortMode, CodegenError, Dir, FnGen, Value, bool_type, gen_assert, gen_block, gen_expr,
    gen_unwinding, same_type,
};
use roop_syntax::{Block, Expr};

/// A Janus loop that undoes itself. Whatever iteration fails, the finished
/// iterations are undone by the loop's own inverse, which runs `step` and
/// `body` backward until the entry condition holds again.
pub fn unwind_from(
    g: &mut FnGen,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let (start, first, second, stop) = match dir {
        Dir::Forward => (entry, body, step, until),
        Dir::Backward => (until, body, step, entry),
    };
    let (head, back, exit) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
    let (undo_first, undo_second, undo_loop) = (g.fresh("L"), g.fresh("L"), g.fresh("L"));
    let outer = g.abort.clone();

    let on_entry = gen_expr(g, start)?;
    gen_assert(g, &on_entry)?;
    g.emit(&format!("br label %{head}"));
    g.label(&head);
    gen_unwinding(g, first, dir, &undo_loop)?;
    g.abort = AbortMode::Label(undo_first.clone());
    let done = gen_expr(g, stop)?;
    same_type(&bool_type(), &done.ty)?;
    g.emit(&format!("br i1 {}, label %{exit}, label %{back}", done.reg));
    g.label(&back);
    gen_unwinding(g, second, dir, &undo_first)?;
    g.abort = AbortMode::Label(undo_second.clone());
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

    g.abort = AbortMode::Trap;
    g.label(&undo_second);
    gen_block(g, second, dir.flip())?;
    g.emit(&format!("br label %{undo_first}"));
    g.label(&undo_first);
    gen_block(g, first, dir.flip())?;
    g.emit(&format!("br label %{undo_loop}"));
    let (undo_step, undo_done) = (g.fresh("L"), g.fresh("L"));
    g.label(&undo_loop);
    let at_start = gen_expr(g, start)?;
    g.emit(&format!(
        "br i1 {}, label %{undo_done}, label %{undo_step}",
        at_start.reg
    ));
    g.label(&undo_step);
    gen_block(g, second, dir.flip())?;
    gen_block(g, first, dir.flip())?;
    g.emit(&format!("br label %{undo_loop}"));
    g.label(&undo_done);
    g.emit(&format!("br label %{fail}"));
    g.abort = outer;
    g.label(&exit);
    Ok(())
}

use crate::{AbortMode, CodegenError, Dir, FnGen, gen_stmt, gen_unwinding_stmt};
use roop_syntax::Block;

/// Runs the block so that a failure undoes everything it already did and then
/// jumps to `fail`. This is the dagger of the executed prefix: the inverse of
/// each finished statement, newest first, which is exactly reverse execution.
/// A statement that fails halfway cleans up after itself, so only finished
/// statements are undone here.
pub fn gen_unwinding(
    g: &mut FnGen,
    block: &Block,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    let stmts: Vec<_> = match dir {
        Dir::Forward => block.stmts.iter().collect(),
        Dir::Backward => block.stmts.iter().rev().collect(),
    };
    let labels: Vec<String> = stmts.iter().map(|_| g.fresh("L")).collect();
    let done = g.fresh("L");
    let outer = g.abort.clone();
    for (stmt, label) in stmts.iter().zip(&labels) {
        g.abort = AbortMode::Label(label.clone());
        let result = gen_unwinding_stmt(g, stmt, dir, label);
        if result.is_err() {
            g.abort = outer;
            return result;
        }
    }
    g.emit(&format!("br label %{done}"));
    g.abort = AbortMode::Trap;
    for k in (0..stmts.len()).rev() {
        g.label(&labels[k]);
        if k == 0 {
            g.emit(&format!("br label %{fail}"));
        } else {
            gen_stmt(g, stmts[k - 1], dir.flip())?;
            g.emit(&format!("br label %{}", labels[k - 1]));
        }
    }
    g.abort = outer;
    g.label(&done);
    Ok(())
}

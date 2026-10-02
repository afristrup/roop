use crate::{CodegenError, Dir, FnGen, declare_ancilla, gen_place, gen_unwinding};
use roop_syntax::{Stmt, StmtKind};

/// Statements that only scope a name or a history around a block: the
/// block does the unwinding, the scope just disappears.
pub fn unwind_scope(
    g: &mut FnGen,
    stmt: &Stmt,
    dir: Dir,
    fail: &str,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => {
            let slot = declare_ancilla(g, ty, init)?;
            g.vars.push((name.clone(), slot));
            let result = gen_unwinding(g, body, dir, fail);
            g.vars.pop();
            result
        }
        StmtKind::Borrow { name, source, body } => {
            let slot = gen_place(g, source)?;
            g.vars.push((name.clone(), slot));
            let result = gen_unwinding(g, body, dir, fail);
            g.vars.pop();
            result
        }
        StmtKind::Logged { history, body } => {
            g.logged.push(history.clone());
            let result = gen_unwinding(g, body, dir, fail);
            g.logged.pop();
            result
        }
        StmtKind::Block(body) => gen_unwinding(g, body, dir, fail),
        _ => Err(CodegenError::InvalidOperand("not a scoping statement")),
    }
}

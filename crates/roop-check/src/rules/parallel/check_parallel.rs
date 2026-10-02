use crate::{CheckError, check_parallel_body};
use roop_syntax::{Attr, Stmt, StmtKind, counted_loop};

pub fn check_parallel(stmt: &Stmt) -> Result<(), CheckError> {
    if !stmt
        .attrs
        .iter()
        .any(|a| matches!(a, Attr::Parallel { .. }))
    {
        return Ok(());
    }
    let StmtKind::From {
        entry,
        body,
        step,
        until,
    } = &stmt.kind
    else {
        return Err(CheckError::ParallelNotLoop { span: stmt.span });
    };
    let counted = counted_loop(entry, step, until)
        .ok_or(CheckError::ParallelLoopShape { span: stmt.span })?;
    check_parallel_body(counted.var, counted.lo, counted.hi, body, stmt.span)
}

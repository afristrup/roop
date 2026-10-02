use roop_syntax::{Attr, Block, Expr, Stmt, StmtKind, Target, counted_loop};

/// A statement that is a `#[parallel]` counted loop.
pub struct ParLoop<'a> {
    pub var: &'a str,
    pub lo: &'a Expr,
    pub hi: &'a Expr,
    pub step: i64,
    pub body: &'a Block,
    pub target: Option<Target>,
}

pub fn par_loop(stmt: &Stmt) -> Option<ParLoop<'_>> {
    let [Attr::Parallel { target }] = stmt.attrs.as_slice() else {
        return None;
    };
    let StmtKind::From {
        entry,
        body,
        step,
        until,
    } = &stmt.kind
    else {
        return None;
    };
    let counted = counted_loop(entry, step, until)?;
    Some(ParLoop {
        var: counted.var,
        lo: counted.lo,
        hi: counted.hi,
        step: counted.step,
        body,
        target: *target,
    })
}

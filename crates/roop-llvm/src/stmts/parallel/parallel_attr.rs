use roop_syntax::{Attr, Stmt, Target};

/// `None` when the statement is not parallel, `Some(None)` for a bare
/// `#[parallel]` the compiler places, `Some(Some(t))` when it names a target.
pub fn parallel_attr(stmt: &Stmt) -> Option<Option<Target>> {
    stmt.attrs.iter().find_map(|attr| match attr {
        Attr::Parallel { target } => Some(*target),
        Attr::Concurrent => None,
    })
}

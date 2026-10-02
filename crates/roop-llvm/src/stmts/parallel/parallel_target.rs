use roop_syntax::{Attr, Stmt, Target};

/// The target of a `#[parallel]` statement; `Cpu` until the optimizer picks.
pub fn parallel_target(stmt: &Stmt) -> Option<Target> {
    stmt.attrs.iter().map(|Attr::Parallel { target }| target.unwrap_or(Target::Cpu)).next()
}

use roop_syntax::{Attr, Stmt};

pub fn is_task(stmt: &Stmt) -> bool {
    stmt.attrs.contains(&Attr::Concurrent)
}

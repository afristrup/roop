use roop_syntax::{Attr, Stmt};

/// `auto ` before an ancilla that the compiler lets go of.
pub fn auto_prefix(stmt: &Stmt) -> &'static str {
    if stmt.attrs.contains(&Attr::Auto) {
        "auto "
    } else {
        ""
    }
}

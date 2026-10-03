use roop_syntax::{Attr, Stmt};

/// `'name: ` before a statement that has a label.
pub fn label_prefix(stmt: &Stmt) -> String {
    stmt.attrs
        .iter()
        .find_map(|attr| match attr {
            Attr::Label(name) => Some(format!("'{name}: ")),
            _ => None,
        })
        .unwrap_or_default()
}

use roop_syntax::{Attr, Stmt};

/// `auto ` or `auto<'region> ` before an ancilla that the compiler lets go of.
pub fn auto_prefix(stmt: &Stmt) -> String {
    stmt.attrs
        .iter()
        .find_map(|attr| match attr {
            Attr::Auto { region: None } => Some("auto ".to_string()),
            Attr::Auto { region: Some(r) } => Some(format!("auto<'{r}> ")),
            _ => None,
        })
        .unwrap_or_default()
}

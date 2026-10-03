use crate::Doc;

/// Whether the layout holds a newline that cannot be avoided.
pub fn has_hard(doc: &Doc) -> bool {
    match doc {
        Doc::HardLine => true,
        Doc::Concat(parts) => parts.iter().any(has_hard),
        Doc::Nest(inner) | Doc::Group(inner) => has_hard(inner),
        _ => false,
    }
}

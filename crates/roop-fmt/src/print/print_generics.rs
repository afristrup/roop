use crate::Doc;

/// `<N, M>`, or nothing without parameters. It never breaks, so the lines
/// of a signature give way in its parameters first.
pub fn print_generics(items: Vec<Doc>) -> Doc {
    if items.is_empty() {
        return Doc::nothing();
    }
    Doc::concat(vec![
        Doc::text("<"),
        Doc::join(items, Doc::text(", ")),
        Doc::text(">"),
    ])
}

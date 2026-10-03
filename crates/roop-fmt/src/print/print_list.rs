use crate::Doc;

/// `open item, item close`, one item to a line with a trailing comma when it
/// does not fit. `spaced` puts a space inside the delimiters when it fits.
pub fn print_list(open: &str, items: Vec<Doc>, close: &str, spaced: bool) -> Doc {
    if items.is_empty() {
        return Doc::text(format!("{open}{close}"));
    }
    let edge = if spaced { Doc::Line } else { Doc::SoftLine };
    let sep = Doc::concat(vec![Doc::text(","), Doc::Line]);
    Doc::group(Doc::concat(vec![
        Doc::text(open),
        Doc::nest(Doc::concat(vec![
            edge.clone(),
            Doc::join(items, sep),
            Doc::IfBreak(","),
        ])),
        edge,
        Doc::text(close),
    ]))
}

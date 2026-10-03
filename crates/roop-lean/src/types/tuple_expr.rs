/// `()`, the value itself, or a right-nested pair.
pub fn tuple_expr(items: &[String]) -> String {
    match items {
        [] => "()".into(),
        [only] => only.clone(),
        _ => format!("({})", items.join(", ")),
    }
}

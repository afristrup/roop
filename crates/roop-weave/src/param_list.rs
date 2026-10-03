/// A parameter or argument list, one to a line, as the generated code writes it.
pub fn param_list(items: &[String]) -> String {
    items.iter().map(|item| format!("    {item},\n")).collect()
}

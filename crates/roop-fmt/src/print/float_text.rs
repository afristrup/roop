/// A float as the lexer reads it back: digits, a point, digits.
pub fn float_text(value: f64) -> String {
    let text = format!("{value}");
    if text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

use roop_syntax::Pattern;

pub fn print_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Int(n) => n.to_string(),
        Pattern::Bool(b) => b.to_string(),
        Pattern::Variant(e, v) => format!("{e}::{v}"),
        Pattern::Wildcard => "_".into(),
    }
}

use crate::{LeanError, esc};
use roop_syntax::Pattern;

pub fn pattern_test(scrutinee: &str, pattern: &Pattern) -> Result<String, LeanError> {
    Ok(match pattern {
        Pattern::Wildcard => "true".into(),
        Pattern::Int(i) => format!("({scrutinee} == (({i}) : Roop.I64))"),
        Pattern::Bool(b) => format!("({scrutinee} == {b})"),
        Pattern::Variant(e, v) => format!("({scrutinee} == {}.{})", esc(e), esc(v)),
    })
}

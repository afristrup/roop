use super::{CheckError, Enums};
use roop_syntax::{MatchArm, Pattern, Span};
use std::collections::HashSet;

/// A reversible match must be total: a wildcard, both bools, or every
/// variant of the matched enum.
pub fn check_exhaustive(enums: &Enums, arms: &[MatchArm], span: Span) -> Result<(), CheckError> {
    let patterns: Vec<&Pattern> = arms.iter().map(|arm| &arm.pattern).collect();
    let total = patterns.contains(&&Pattern::Wildcard)
        || (patterns.contains(&&Pattern::Bool(true)) && patterns.contains(&&Pattern::Bool(false)))
        || covers_enum(enums, &patterns);
    if total {
        Ok(())
    } else {
        Err(CheckError::NonExhaustiveMatch { span })
    }
}

fn covers_enum(enums: &Enums, patterns: &[&Pattern]) -> bool {
    let Some(Pattern::Variant(enum_name, _)) = patterns.first() else {
        return false;
    };
    let named: HashSet<&str> = patterns
        .iter()
        .filter_map(|p| match p {
            Pattern::Variant(e, v) if e == enum_name => Some(v.as_str()),
            _ => None,
        })
        .collect();
    enums
        .get(enum_name.as_str())
        .is_some_and(|def| def.variants.iter().all(|v| named.contains(v.as_str())))
}

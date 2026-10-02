use crate::lean_type;
use roop_syntax::Type;

/// `Unit`, the type itself, or right-nested pairs, for a function's results.
pub fn tuple_type(types: &[Type]) -> String {
    match types {
        [] => "Unit".into(),
        [only] => lean_type(only),
        [first, rest @ ..] => format!("({} \u{d7} {})", lean_type(first), tuple_type(rest)),
    }
}

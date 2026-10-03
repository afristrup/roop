use crate::{Out, esc};
use roop_syntax::Type;

/// Opens the state tuple `s` into mutable variables.
pub fn unpack_state(out: &mut Out, state: &[(String, Type)]) {
    let names: Vec<String> = state.iter().map(|(n, _)| esc(n)).collect();
    match names.as_slice() {
        [] => {}
        [only] => out.line(&format!("let mut {only} := __st")),
        many => {
            out.line(&format!("let ({}) := __st", many.join(", ")));
            for name in many {
                out.line(&format!("let mut {name} := {name}"));
            }
        }
    }
}

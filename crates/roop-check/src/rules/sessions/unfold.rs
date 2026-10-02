use crate::substitute;
use roop_syntax::Behaviour;

/// Unrolls `rec x { b }` to `b` with itself in place of `x`, until the
/// behaviour starts with a choice or is finished. Recursion is guarded, so this
/// ends.
pub fn unfold(behaviour: &Behaviour) -> Behaviour {
    match behaviour {
        Behaviour::Rec(name, body) => unfold(&substitute(body, name, behaviour)),
        other => other.clone(),
    }
}

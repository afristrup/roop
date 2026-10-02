use crate::{FnGen, Slot};
use roop_check::BodyEffects;

/// The visible variables a loop body touches, each paired with whether the
/// body writes it, excluding the induction variable. Read-only buffers are
/// not copied back.
pub fn device_buffers(g: &FnGen, var: &str, effects: &BodyEffects) -> Vec<(String, Slot, bool)> {
    let mut chosen: Vec<(String, Slot, bool)> = Vec::new();
    for (name, slot) in g.vars.iter().rev() {
        let used = effects.reads.contains(name) || effects.writes.contains(name);
        if name != var && used && !chosen.iter().any(|(n, _, _)| n == name) {
            chosen.push((name.clone(), slot.clone(), effects.writes.contains(name)));
        }
    }
    chosen.reverse();
    chosen
}

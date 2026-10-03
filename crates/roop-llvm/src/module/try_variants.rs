use crate::Ctx;
use roop_check::calls_in;
use std::collections::BTreeSet;

/// The failure-atomic variants the program needs, as `(function, inverse)`.
/// Calls inside a `try` that keeps its outcome need them, and so does
/// everything those variants call, since a failing callee must undo itself.
pub fn try_variants(ctx: &Ctx) -> BTreeSet<(String, bool)> {
    let mut needed: BTreeSet<(String, bool)> = BTreeSet::new();
    let mut work: Vec<(String, bool)> = Vec::new();
    let mut want = |callee: &str, is_uncall: bool, backward: bool, work: &mut Vec<_>| {
        let key = (callee.to_string(), is_uncall != backward);
        if needed.insert(key.clone()) {
            work.push(key);
        }
    };
    for def in ctx.fns.values() {
        // An irreversible function has no backward run, but its forward run may
        // still contain a `try`.
        let directions: &[bool] = if ctx.irreversible.contains(def.name.as_str()) {
            &[false]
        } else {
            &[false, true]
        };
        for &backward in directions {
            for (callee, is_uncall) in calls_in(&def.body, true) {
                want(&callee, is_uncall, backward, &mut work);
            }
        }
    }
    while let Some((name, inverse)) = work.pop() {
        let Some(def) = ctx.fns.get(name.as_str()) else {
            continue;
        };
        for (callee, is_uncall) in calls_in(&def.body, false) {
            want(&callee, is_uncall, inverse, &mut work);
        }
    }
    needed
}

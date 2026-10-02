use crate::find_non_unwindable;
use roop_syntax::{Item, Program};
use std::collections::HashSet;

/// Functions that cannot promise to leave their arguments untouched when they
/// fail: those with code a rollback could not undo, and every caller of one.
pub fn non_atomic_fns(program: &Program) -> HashSet<&str> {
    let mut blocked: HashSet<&str> = HashSet::new();
    loop {
        let before = blocked.len();
        for item in &program.items {
            if let Item::Fn(f) = item
                && (f.irreversible || find_non_unwindable(&f.body, false, &blocked).is_some())
            {
                blocked.insert(f.name.as_str());
            }
        }
        if blocked.len() == before {
            return blocked;
        }
    }
}

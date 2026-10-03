use crate::has_keep_effect;
use roop_syntax::{Item, Program};
use std::collections::HashSet;

/// The functions that keep values, and everything that calls one.
pub fn keeping_fns(program: &Program) -> HashSet<&str> {
    let mut found: HashSet<&str> = HashSet::new();
    loop {
        let before = found.len();
        for item in &program.items {
            if let Item::Fn(f) = item
                && f.body.stmts.iter().any(|s| has_keep_effect(s, &found))
            {
                found.insert(f.name.as_str());
            }
        }
        if found.len() == before {
            return found;
        }
    }
}

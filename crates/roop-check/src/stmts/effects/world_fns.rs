use roop_syntax::{Item, Program};
use std::collections::HashSet;

use crate::{calls_in, find_world_call};

/// The functions that change the world outside the program: the extern ones
/// declared `world`, and everything that calls one.
pub fn world_fns(program: &Program) -> HashSet<&str> {
    let mut found: HashSet<&str> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) if f.external && f.world => Some(f.name.as_str()),
            _ => None,
        })
        .collect();
    loop {
        let before = found.len();
        for item in &program.items {
            if let Item::Fn(f) = item
                && (find_world_call(&f.body, &found).is_some()
                    || calls_in(&f.body, false)
                        .iter()
                        .any(|(callee, _)| found.contains(callee.as_str())))
            {
                found.insert(f.name.as_str());
            }
        }
        if found.len() == before {
            return found;
        }
    }
}

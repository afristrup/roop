use crate::is_irreversible_fn;
use roop_syntax::{Item, Program};
use std::collections::HashSet;

pub fn irreversible_fns(program: &Program) -> HashSet<&str> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) if is_irreversible_fn(f) => Some(f.name.as_str()),
            _ => None,
        })
        .collect()
}

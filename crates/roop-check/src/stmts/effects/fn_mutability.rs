use roop_syntax::{Item, Program, Type};
use std::collections::HashMap;

/// For each function, which parameters it may write: the `&mut` ones.
pub type Mutability<'a> = HashMap<&'a str, Vec<bool>>;

pub fn fn_mutability(program: &Program) -> Mutability<'_> {
    program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) => Some((
                f.name.as_str(),
                f.params
                    .iter()
                    .map(|p| matches!(p.ty, Type::Ref { mutable: true, .. }))
                    .collect(),
            )),
            _ => None,
        })
        .collect()
}

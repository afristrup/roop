use crate::{BennettError, build_bennett};
use roop_syntax::{FnDef, Item, Program};

/// Writes out every `bennett fn name = target;`.
pub fn expand_bennett(program: &Program) -> Result<Program, BennettError> {
    let find = |name: &str| -> Option<&FnDef> {
        program.items.iter().find_map(|item| match item {
            Item::Fn(f) if f.name == name => Some(f),
            _ => None,
        })
    };
    let mut items = Vec::new();
    for item in &program.items {
        let Item::Fn(f) = item else {
            items.push(item.clone());
            continue;
        };
        let Some(target) = &f.bennett else {
            items.push(item.clone());
            continue;
        };
        let def = find(target).ok_or_else(|| BennettError::UnknownTarget {
            name: f.name.clone(),
            target: target.clone(),
        })?;
        items.push(Item::Fn(build_bennett(program, &f.name, f.public, def)?));
    }
    Ok(Program { items })
}

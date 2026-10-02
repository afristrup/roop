use crate::{GenericError, Instances, Rewriter};
use roop_syntax::{Item, Program, walk_item};
use std::collections::HashMap;

/// Replaces every generic function by one instance per set of lengths it is
/// called with, so later stages only see concrete types. Generic functions
/// nothing calls disappear.
pub fn monomorphize(program: &Program) -> Result<Program, GenericError> {
    let defs = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) if !f.generics.is_empty() => Some((f.name.as_str(), f)),
            _ => None,
        })
        .collect();
    let mut instances = Instances::new(defs);
    let mut items = Vec::new();
    let no_lengths = HashMap::new();
    for item in &program.items {
        if matches!(item, Item::Fn(f) if !f.generics.is_empty()) {
            continue;
        }
        let mut item = item.clone();
        rewrite(&mut item, &no_lengths, &mut instances)?;
        items.push(item);
    }
    while let Some((callee, lengths, name)) = instances.queue.pop() {
        let mut def = instances.defs[callee.as_str()].clone();
        let env: HashMap<String, i64> = def.generics.drain(..).zip(lengths).collect();
        def.name = name;
        let mut item = Item::Fn(def);
        rewrite(&mut item, &env, &mut instances)?;
        items.push(item);
    }
    Ok(Program { items })
}

fn rewrite(
    item: &mut Item,
    env: &HashMap<String, i64>,
    instances: &mut Instances,
) -> Result<(), GenericError> {
    let mut rewriter = Rewriter {
        env,
        instances,
        error: None,
    };
    walk_item(&mut rewriter, item);
    rewriter.error.map_or(Ok(()), Err)
}

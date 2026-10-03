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
            Item::Fn(f) if !f.generics.is_empty() && !f.external => Some((f.name.as_str(), f)),
            _ => None,
        })
        .collect();
    let mut instances = Instances::new(defs);
    let mut items = Vec::new();
    let no_lengths = HashMap::new();
    for item in &program.items {
        if matches!(item, Item::Fn(f) if !f.generics.is_empty() && !f.external) {
            continue;
        }
        let mut item = item.clone();
        if let Item::Fn(f) = &mut item
            && f.external
        {
            erase_lengths(f);
        }
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

/// An extern function takes pointers, whatever the lengths, so its generic
/// lengths are dropped and its array types stand for any length.
fn erase_lengths(f: &mut roop_syntax::FnDef) {
    f.generics.clear();
    for param in &mut f.params {
        param.ty = erased(&param.ty);
    }
}

fn erased(ty: &roop_syntax::Type) -> roop_syntax::Type {
    use roop_syntax::Type;
    match ty {
        Type::Param {
            elem, stack: true, ..
        } => Type::Stack(Box::new(erased(elem)), 0),
        Type::Param { elem, .. } => Type::Array(Box::new(erased(elem)), 0),
        Type::Ref { mutable, inner } => Type::Ref {
            mutable: *mutable,
            inner: Box::new(erased(inner)),
        },
        Type::Array(elem, n) => Type::Array(Box::new(erased(elem)), *n),
        Type::Stack(elem, n) => Type::Stack(Box::new(erased(elem)), *n),
        other => other.clone(),
    }
}

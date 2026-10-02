use roop_syntax::{StructDef, Type};
use std::collections::{HashMap, HashSet};

/// The structs with an `f64` somewhere inside: directly in a field, or in a
/// struct, array or stack that a field holds.
pub fn float_structs(structs: &HashMap<&str, &StructDef>) -> HashSet<String> {
    let mut found: HashSet<String> = HashSet::new();
    loop {
        let before = found.len();
        for (name, def) in structs {
            if def.fields.iter().any(|f| holds_float(&f.ty, &found)) {
                found.insert(name.to_string());
            }
        }
        if found.len() == before {
            return found;
        }
    }
}

fn holds_float(ty: &Type, structs: &HashSet<String>) -> bool {
    match ty {
        Type::Named(name) => name == "f64" || structs.contains(name),
        Type::Ref { inner, .. }
        | Type::Array(inner, _)
        | Type::Stack(inner, _)
        | Type::Param { elem: inner, .. } => holds_float(inner, structs),
    }
}

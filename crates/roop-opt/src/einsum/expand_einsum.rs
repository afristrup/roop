use crate::{EinsumError, build_einsum};
use roop_syntax::{Item, Program};

/// Writes out every `einsum fn`.
pub fn expand_einsum(program: &Program) -> Result<Program, EinsumError> {
    let mut items = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) if f.einsum.is_some() => items.push(Item::Fn(build_einsum(f)?)),
            other => items.push(other.clone()),
        }
    }
    Ok(Program { items })
}

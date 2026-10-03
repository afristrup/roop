use crate::{AutoError, expand_block};
use roop_syntax::{Item, Program};

/// Writes out the keeps of every `auto ancilla`, placed where its block ends,
/// the way a compiler ends a lifetime. A loop body that declares one lets go of
/// it each round.
pub fn expand_auto(program: &Program) -> Result<Program, AutoError> {
    let mut items = program.items.clone();
    for item in &mut items {
        if let Item::Fn(f) = item {
            expand_block(&mut f.body)?;
        }
    }
    Ok(Program { items })
}

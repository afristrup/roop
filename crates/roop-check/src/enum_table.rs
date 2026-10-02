use super::{CheckError, Enums};
use roop_syntax::{Item, Program};
use std::collections::HashSet;

pub fn enum_table(program: &Program) -> Result<Enums<'_>, CheckError> {
    let mut table = Enums::new();
    for item in &program.items {
        let Item::Enum(def) = item else { continue };
        let mut seen = HashSet::new();
        if let Some(dup) = def.variants.iter().find(|v| !seen.insert(v.as_str())) {
            return Err(CheckError::DuplicateVariant {
                enum_name: def.name.clone(),
                variant: dup.clone(),
                span: def.span,
            });
        }
        table.insert(&def.name, def);
    }
    Ok(table)
}

use crate::{esc, lean_type};
use roop_syntax::StructDef;

pub fn lean_struct(def: &StructDef) -> String {
    let mut text = format!("structure {} where\n", esc(&def.name));
    for field in &def.fields {
        text.push_str(&format!(
            "  {} : {}\n",
            esc(&field.name),
            lean_type(&field.ty)
        ));
    }
    text.push('\n');
    text
}

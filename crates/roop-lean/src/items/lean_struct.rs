use crate::{esc, esc_ty, lean_type};
use roop_syntax::StructDef;

pub fn lean_struct(def: &StructDef) -> String {
    let mut text = format!("@[ext] structure {} where\n", esc_ty(&def.name));
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

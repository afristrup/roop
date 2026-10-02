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
    let ty = esc_ty(&def.name);
    let fields: Vec<String> = def
        .fields
        .iter()
        .map(|f| format!("(s).{}", esc(&f.name)))
        .collect();
    // A struct rebuilt from its own fields is itself.
    text.push_str(&format!(
        "\n@[simp] theorem {ty}.eta_fields (s : {ty}) : {ty}.mk {} = s := rfl\n\n",
        fields.join(" ")
    ));
    text
}

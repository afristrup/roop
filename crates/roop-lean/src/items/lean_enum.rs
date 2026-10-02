use crate::esc;
use roop_syntax::EnumDef;

pub fn lean_enum(def: &EnumDef) -> String {
    let mut text = format!("inductive {} where\n", esc(&def.name));
    for variant in &def.variants {
        text.push_str(&format!("  | {}\n", esc(variant)));
    }
    text.push_str("  deriving DecidableEq, BEq, Repr\n\n");
    text
}

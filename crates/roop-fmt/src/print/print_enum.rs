use crate::{Doc, print_list};
use roop_syntax::EnumDef;

pub fn print_enum(def: &EnumDef) -> Doc {
    let public = if def.public { "pub " } else { "" };
    let variants = def.variants.iter().map(Doc::text).collect();
    Doc::concat(vec![
        Doc::text(format!("{public}enum {} ", def.name)),
        print_list("{", variants, "}", true),
    ])
}

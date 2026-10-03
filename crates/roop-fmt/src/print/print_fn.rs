use crate::{Ctx, Doc, print_block, print_generics, print_params};
use roop_syntax::FnDef;

pub fn print_fn(ctx: &Ctx, f: &FnDef) -> Doc {
    let public = if f.public { "pub " } else { "" };
    let irrev = if f.irreversible { "irrev " } else { "" };
    Doc::concat(vec![
        Doc::text(format!("{public}{irrev}fn {}", f.name)),
        print_generics(f.generics.iter().map(Doc::text).collect()),
        print_params(&f.params),
        Doc::text(" "),
        print_block(ctx, &f.body, false),
    ])
}

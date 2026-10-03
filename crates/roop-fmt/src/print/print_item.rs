use crate::{Ctx, Doc, print_enum, print_fn, print_session, print_struct, print_use};
use roop_syntax::Item;

pub fn print_item(ctx: &Ctx, item: &Item) -> Doc {
    match item {
        Item::Mod(name) => Doc::text(format!("mod {name};")),
        Item::Use(decl) => print_use(decl),
        Item::Enum(def) => print_enum(def),
        Item::Struct(def) => print_struct(ctx, def),
        Item::Fn(f) => print_fn(ctx, f),
        Item::Session(def) => print_session(def),
    }
}

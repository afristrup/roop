use roop_syntax::Item;

/// The name and visibility of a definition; `mod` and `use` define nothing.
pub fn item_name(item: &Item) -> Option<(&str, bool)> {
    match item {
        Item::Fn(f) => Some((&f.name, f.public)),
        Item::Struct(s) => Some((&s.name, s.public)),
        Item::Enum(e) => Some((&e.name, e.public)),
        Item::Mod(_) | Item::Use(_) => None,
    }
}

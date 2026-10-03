use crate::global_name;
use roop_syntax::Item;

/// The program-wide name of a definition. An `extern fn` keeps its own name,
/// since that is the symbol the runtime provides.
pub fn item_global(path: &[String], item: &Item, name: &str) -> String {
    match item {
        Item::Fn(f) if f.external => name.to_string(),
        _ => global_name(path, name),
    }
}

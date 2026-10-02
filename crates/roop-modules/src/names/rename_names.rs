use roop_syntax::{Item, Visitor, walk_item};

struct Names<'a>(&'a mut dyn FnMut(&mut String));

impl Visitor for Names<'_> {
    fn name(&mut self, name: &mut String) {
        (self.0)(name);
    }
}

/// Calls `on` for every function, struct or enum name an item refers to, not
/// the name it defines.
pub fn rename_names(item: &mut Item, on: &mut dyn FnMut(&mut String)) {
    walk_item(&mut Names(on), item);
}

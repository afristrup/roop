use crate::{BuildFn, Item, Param, Visitor, walk_block, walk_type};

fn walk_params(v: &mut dyn Visitor, params: &mut [Param]) {
    for param in params {
        walk_type(v, &mut param.ty);
    }
}

fn walk_build(v: &mut dyn Visitor, build: &mut BuildFn) {
    walk_params(v, &mut build.params);
    walk_block(v, &mut build.body);
}

/// Walks what an item's signature and body refer to, not the name it defines.
pub fn walk_item(v: &mut dyn Visitor, item: &mut Item) {
    match item {
        Item::Fn(f) => {
            walk_params(v, &mut f.params);
            walk_block(v, &mut f.body);
        }
        Item::Struct(s) => {
            for field in &mut s.fields {
                walk_type(v, &mut field.ty);
            }
            for build in [&mut s.build, &mut s.unbuild].into_iter().flatten() {
                walk_build(v, build);
            }
        }
        Item::Enum(_) | Item::Session(_) | Item::Mod(_) | Item::Use(_) => {}
    }
}

use crate::{OnName, visit_block, visit_type};
use roop_syntax::{BuildFn, Item, Param};

fn visit_params(params: &mut [Param], on: OnName) {
    for param in params {
        visit_type(&mut param.ty, on);
    }
}

fn visit_build(build: &mut BuildFn, on: OnName) {
    visit_params(&mut build.params, on);
    visit_block(&mut build.body, on);
}

/// Visits every name an item's signature and body refer to, not the name it
/// defines.
pub fn visit_item(item: &mut Item, on: OnName) {
    match item {
        Item::Fn(f) => {
            visit_params(&mut f.params, on);
            visit_block(&mut f.body, on);
        }
        Item::Struct(s) => {
            for field in &mut s.fields {
                visit_type(&mut field.ty, on);
            }
            for build in [&mut s.build, &mut s.unbuild].into_iter().flatten() {
                visit_build(build, on);
            }
        }
        Item::Enum(_) | Item::Mod(_) | Item::Use(_) => {}
    }
}

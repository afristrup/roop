use crate::fuse_block;
use roop_syntax::{BuildFn, Item, Program};

/// Merges adjacent `#[parallel]` loops that share an iteration space into
/// single loops (single kernels on a GPU) wherever that stays legal.
pub fn fuse_parallel(program: &Program) -> Program {
    let ctor = |c: &Option<BuildFn>| {
        c.as_ref().map(|c| BuildFn {
            params: c.params.clone(),
            body: fuse_block(&c.body),
        })
    };
    let items = program
        .items
        .iter()
        .map(|item| match item {
            Item::Fn(f) => {
                let mut f = f.clone();
                f.body = fuse_block(&f.body);
                Item::Fn(f)
            }
            Item::Struct(def) => {
                let mut def = def.clone();
                def.build = ctor(&def.build);
                def.unbuild = ctor(&def.unbuild);
                Item::Struct(def)
            }
            other => other.clone(),
        })
        .collect();
    Program { items }
}

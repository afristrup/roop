use crate::float_structs;
use roop_syntax::{EnumDef, FnDef, Item, Program, StructDef};
use std::collections::{HashMap, HashSet};

/// The program's declarations, and which functions have been translated so
/// far (a call is only translatable when its callee was).
pub struct Ctx<'a> {
    pub structs: HashMap<&'a str, &'a StructDef>,
    pub enums: HashMap<&'a str, &'a EnumDef>,
    pub fns: HashMap<&'a str, &'a FnDef>,
    pub translated: HashSet<String>,
    /// Structs with an `f64` inside, whose arithmetic is not exact.
    pub float_structs: HashSet<String>,
}

impl<'a> Ctx<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut ctx = Ctx {
            structs: HashMap::new(),
            enums: HashMap::new(),
            fns: HashMap::new(),
            translated: HashSet::new(),
            float_structs: HashSet::new(),
        };
        for item in &program.items {
            match item {
                Item::Struct(def) => drop(ctx.structs.insert(&def.name, def)),
                Item::Enum(def) => drop(ctx.enums.insert(&def.name, def)),
                Item::Fn(def) => drop(ctx.fns.insert(&def.name, def)),
                Item::Mod(_) | Item::Use(_) | Item::Session(_) => {}
            }
        }
        ctx.float_structs = float_structs(&ctx.structs);
        ctx
    }
}

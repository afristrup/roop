use crate::Options;
use roop_syntax::{EnumDef, FnDef, Item, Program, StructDef};
use std::collections::HashMap;

pub struct Ctx<'a> {
    pub structs: HashMap<&'a str, &'a StructDef>,
    pub enums: HashMap<&'a str, &'a EnumDef>,
    pub fns: HashMap<&'a str, &'a FnDef>,
    pub options: &'a Options,
}

impl<'a> Ctx<'a> {
    pub fn new(program: &'a Program, options: &'a Options) -> Self {
        let mut ctx = Ctx {
            structs: HashMap::new(),
            enums: HashMap::new(),
            fns: HashMap::new(),
            options,
        };
        for item in &program.items {
            match item {
                Item::Struct(def) => ctx.structs.insert(&def.name, def).map(drop),
                Item::Enum(def) => ctx.enums.insert(&def.name, def).map(drop),
                Item::Fn(def) => ctx.fns.insert(&def.name, def).map(drop),
                Item::Mod(_) | Item::Use(_) => None,
            };
        }
        ctx
    }
}

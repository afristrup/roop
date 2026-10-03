use crate::Options;
use roop_check::{
    Mutability, fn_mutability, irreversible_fns, is_irreversible_fn, keeping_fns, world_fns,
};
use roop_syntax::{EnumDef, FnDef, Item, Program, StructDef};
use std::collections::HashMap;

pub struct Ctx<'a> {
    pub structs: HashMap<&'a str, &'a StructDef>,
    pub enums: HashMap<&'a str, &'a EnumDef>,
    pub fns: HashMap<&'a str, &'a FnDef>,
    pub options: &'a Options,
    /// Functions with irreversible code, which have no inverse.
    pub irreversible: std::collections::HashSet<&'a str>,
    /// Which parameters of each function it may write.
    pub mutability: Mutability<'a>,
    /// Functions that do more than write their arguments: they change the world,
    /// keep values or are irreversible.
    pub effectful: std::collections::HashSet<&'a str>,
}

impl<'a> Ctx<'a> {
    pub fn new(program: &'a Program, options: &'a Options) -> Self {
        let mut ctx = Ctx {
            structs: HashMap::new(),
            enums: HashMap::new(),
            fns: HashMap::new(),
            options,
            irreversible: std::collections::HashSet::new(),
            mutability: fn_mutability(program),
            effectful: world_fns(program)
                .into_iter()
                .chain(keeping_fns(program))
                .chain(irreversible_fns(program))
                .collect(),
        };
        for item in &program.items {
            match item {
                Item::Struct(def) => ctx.structs.insert(&def.name, def).map(drop),
                Item::Enum(def) => ctx.enums.insert(&def.name, def).map(drop),
                Item::Fn(def) => ctx.fns.insert(&def.name, def).map(drop),
                Item::Mod(_) | Item::Use(_) | Item::Session(_) => None,
            };
        }
        ctx.irreversible = ctx
            .fns
            .values()
            .filter(|f| is_irreversible_fn(f))
            .map(|f| f.name.as_str())
            .collect();
        ctx
    }
}

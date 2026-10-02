use crate::{EnumDef, FnDef, StructDef};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Mod(String),
    Use(Vec<String>),
    Enum(EnumDef),
    Struct(StructDef),
    Fn(FnDef),
}

use crate::{EnumDef, FnDef, StructDef, UseDecl};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Mod(String),
    Use(UseDecl),
    Enum(EnumDef),
    Struct(StructDef),
    Fn(FnDef),
}

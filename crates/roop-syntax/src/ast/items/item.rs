use crate::{EnumDef, FnDef, SessionDef, StructDef, UseDecl};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Mod(String),
    Use(UseDecl),
    Enum(EnumDef),
    Struct(StructDef),
    Fn(FnDef),
    Session(SessionDef),
}

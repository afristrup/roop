use super::{FnDef, StructDef};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Mod(String),
    Use(Vec<String>),
    Struct(StructDef),
    Fn(FnDef),
}

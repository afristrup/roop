use crate::{Block, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
}

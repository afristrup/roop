use crate::{Block, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    /// Declared `irrev fn`: the whole body is irreversible code.
    pub irreversible: bool,
    pub public: bool,
}

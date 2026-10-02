use super::{Block, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct BuildFn {
    pub params: Vec<Param>,
    pub body: Block,
}

use crate::{Block, Expr, Pattern};

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Block,
    pub exit: Expr,
}

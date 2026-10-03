use crate::Span;
use crate::Stmt;

/// The statements between a pair of braces. The span covers the braces and is
/// zero for blocks the compiler builds.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            stmts: Vec::new(),
            span: Span::from(0..0),
        }
    }
}

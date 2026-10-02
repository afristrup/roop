use crate::Span;
use crate::{Attr, StmtKind};

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub attrs: Vec<Attr>,
    pub kind: StmtKind,
    pub span: Span,
}

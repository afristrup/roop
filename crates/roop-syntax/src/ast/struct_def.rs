use super::{BuildFn, Field};
use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub build: Option<BuildFn>,
    pub unbuild: Option<BuildFn>,
    pub span: Span,
}

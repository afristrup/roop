use crate::{Role, Span};

/// `session S { client: ...; server: ...; }`. With two roles the first is the
/// client and the second the server; with one the server is its dual.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionDef {
    pub name: String,
    pub roles: Vec<Role>,
    pub span: Span,
    pub public: bool,
}

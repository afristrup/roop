/// What a `use` brings in from the module its path names.
#[derive(Clone, Debug, PartialEq)]
pub enum UseShape {
    /// `use a::b::c;` The last segment of the path is the item.
    Single,
    /// `use a::b::*;` Every public item of the module.
    Glob,
    /// `use a::b::{c, d as e};` Some items of the module.
    Group(Vec<(String, Option<String>)>),
}

/// `use a::b::c [as d];`, `use a::b::*;` or `use a::b::{c, d};`, optionally
/// `pub` to re-export what it imports.
#[derive(Clone, Debug, PartialEq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub public: bool,
    pub shape: UseShape,
}

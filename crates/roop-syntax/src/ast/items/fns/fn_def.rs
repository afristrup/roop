use crate::{Block, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct FnDef {
    pub name: String,
    /// Length parameters: `fn f<N>(x: &[i64; N])`.
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub body: Block,
    /// Declared `irrev fn`: the whole body is irreversible code.
    pub irreversible: bool,
    pub public: bool,
    /// Declared `test name { fixtures; body }`: a function over its fixtures,
    /// which `roop test` runs forward and then backward.
    pub test: bool,
    /// Declared `bennett fn name = target;`: the compute, copy, uncompute
    /// version of `target`, which `roop-opt` writes out before the checks.
    pub bennett: Option<String>,
}

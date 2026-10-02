use crate::Module;
use std::collections::BTreeMap;

/// Every loaded module, with the entry module and the `Roop.toml` roots.
#[derive(Debug, Default)]
pub struct Tree {
    pub modules: Vec<Module>,
    pub entry: usize,
    pub roots: BTreeMap<String, usize>,
}

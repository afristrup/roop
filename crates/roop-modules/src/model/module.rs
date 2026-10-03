use roop_syntax::Item;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One `.roop` file: its items and the submodules it declares with `mod`.
#[derive(Debug)]
pub struct Module {
    pub path: Vec<String>,
    pub dir: PathBuf,
    pub items: Vec<Item>,
    pub children: BTreeMap<String, usize>,
    /// The module that declared this one with `mod`; the roots have none.
    pub parent: Option<usize>,
}

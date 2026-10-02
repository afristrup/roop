use crate::ParallelConfig;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The parsed `Roop.toml`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub modules: BTreeMap<String, String>,
    pub parallel: ParallelConfig,
    /// The directory holding `Roop.toml`; module paths are relative to it.
    #[serde(skip)]
    pub root: PathBuf,
}

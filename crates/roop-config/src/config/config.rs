use crate::ParallelConfig;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The parsed `Roop.toml`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub modules: BTreeMap<String, String>,
    pub parallel: ParallelConfig,
}

use serde::Deserialize;

/// The `[world]` table of `Roop.toml`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct WorldConfig {
    /// The most bytes the history of kept values may hold. A `keep` that would
    /// pass it stops the program. Unlimited when absent.
    pub history_limit: Option<u64>,
}

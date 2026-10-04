use serde::Deserialize;

/// The `[checks]` table of `Roop.toml`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ChecksConfig {
    /// When true, an integer addition, subtraction, negation or multiplication
    /// that wraps sets the `roop_overflow` flag, which a C driver reads. The
    /// results do not change. Off by default, at no cost.
    pub overflow: bool,
}

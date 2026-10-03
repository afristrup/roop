use serde::Deserialize;

/// The `[format]` table of `Roop.toml`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct FormatConfig {
    /// The longest line `roop fmt` writes before it wraps.
    pub max_width: usize,
    /// Spaces per level of indentation.
    pub indent: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            max_width: 88,
            indent: 4,
        }
    }
}

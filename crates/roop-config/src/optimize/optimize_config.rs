use serde::Deserialize;

fn yes() -> bool {
    true
}

/// The `[optimize]` table of `Roop.toml`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct OptimizeConfig {
    /// When false, an ancilla that a call made and an `uncall` takes off again is
    /// computed backward, as written, and not just set to zero.
    pub clear_ancillas: bool,
}

impl Default for OptimizeConfig {
    fn default() -> Self {
        OptimizeConfig {
            clear_ancillas: yes(),
        }
    }
}

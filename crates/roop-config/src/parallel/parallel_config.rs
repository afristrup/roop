use serde::Deserialize;

fn yes() -> bool {
    true
}

fn all_targets() -> Vec<String> {
    ["cpu", "metal", "cuda"].map(String::from).to_vec()
}

/// The `[parallel]` table of `Roop.toml`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ParallelConfig {
    /// When false, a bare `#[parallel]` always means CPU threads.
    pub auto: bool,
    /// Targets the program may use. Remove one to opt out of it, including
    /// when a loop asks for it by name.
    pub targets: Vec<String>,
    /// LLVM CPU model, e.g. `apple-m4`. Detected on macOS when absent.
    pub cpu: Option<String>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            auto: yes(),
            targets: all_targets(),
            cpu: None,
        }
    }
}

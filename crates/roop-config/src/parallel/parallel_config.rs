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
    /// When false, matrix products are loops even where the CPU has the SME
    /// matrix unit.
    pub sme: bool,
    /// When false, products of `q12` matrices (weave's einsums) are loops and
    /// not the runtime's integer kernel.
    pub q12: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        ParallelConfig {
            auto: yes(),
            targets: all_targets(),
            cpu: None,
            sme: yes(),
            q12: yes(),
        }
    }
}

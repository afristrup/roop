use crate::{Loader, ModuleError, discover_roots, flatten};
use roop_config::Config;
use roop_syntax::Program;
use std::path::Path;

/// Loads `entry` and every module it reaches through `mod` and `use`, and
/// flattens them into one program.
pub fn load_program(entry: &Path, config: &Config) -> Result<Program, ModuleError> {
    let mut loader = Loader::new(config);
    loader.load_entry(entry)?;
    discover_roots(&mut loader)?;
    flatten(&loader.tree)
}

use crate::{Config, ConfigError, parse_config};
use std::path::Path;

/// Finds the nearest `Roop.toml` at or above `start` and parses it. A project
/// without one gets the defaults.
pub fn load_config(start: &Path) -> Result<Config, ConfigError> {
    for dir in start.ancestors() {
        let path = dir.join("Roop.toml");
        if path.is_file() {
            let origin = path.display().to_string();
            let text =
                std::fs::read_to_string(&path).map_err(|e| ConfigError::Read(origin.clone(), e))?;
            return parse_config(&origin, &text);
        }
    }
    Ok(Config::default())
}

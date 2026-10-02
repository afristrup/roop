use crate::{Config, ConfigError};

const TARGETS: [&str; 3] = ["cpu", "metal", "nvptx"];

/// Parses `Roop.toml` text. `origin` names the file in error messages.
pub fn parse_config(origin: &str, text: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(text).map_err(|e| ConfigError::Parse(origin.into(), e))?;
    match config
        .parallel
        .targets
        .iter()
        .find(|t| !TARGETS.contains(&t.as_str()))
    {
        Some(bad) => Err(ConfigError::UnknownTarget(bad.clone())),
        None => Ok(config),
    }
}

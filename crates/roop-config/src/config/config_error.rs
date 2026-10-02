use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    Read(String, std::io::Error),
    Parse(String, toml::de::Error),
    UnknownTarget(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Read(path, e) => write!(f, "cannot read {path}: {e}"),
            Self::Parse(path, e) => write!(f, "invalid {path}: {e}"),
            Self::UnknownTarget(name) => {
                write!(
                    f,
                    "unknown parallel target `{name}` (expected cpu, metal or nvptx)"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

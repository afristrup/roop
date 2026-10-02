use crate::CliError;
use roop_syntax::Program;
use std::path::Path;

/// Loads `input` and the modules it uses, configured by the nearest
/// `Roop.toml`, with generic functions instantiated.
pub fn load_source(input: &Path, config: &roop_config::Config) -> Result<Program, CliError> {
    let program = roop_modules::load_program(input, config).map_err(CliError::Modules)?;
    roop_opt::monomorphize(&program).map_err(CliError::Generics)
}

/// The nearest `Roop.toml` above `input`.
pub fn config_for(input: &Path) -> Result<roop_config::Config, CliError> {
    let start = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
    roop_config::load_config(start.parent().unwrap_or(Path::new("."))).map_err(CliError::Config)
}

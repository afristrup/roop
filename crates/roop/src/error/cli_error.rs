use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Io(String, std::io::Error),
    Config(roop_config::ConfigError),
    Modules(roop_modules::ModuleError),
    Generics(roop_opt::GenericError),
    Bennett(roop_opt::BennettError),
    Auto(roop_opt::AutoError),
    Einsum(roop_opt::EinsumError),
    Check(roop_check::CheckError),
    Codegen(roop_llvm::CodegenError),
    Format(String, roop_fmt::FmtError),
    Tool(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Usage(text) => write!(f, "{text}"),
            Self::Io(what, e) => write!(f, "{what}: {e}"),
            Self::Config(e) => write!(f, "{e}"),
            Self::Modules(e) => write!(f, "{e}"),
            Self::Generics(e) => write!(f, "{e}"),
            Self::Bennett(e) => write!(f, "{e}"),
            Self::Auto(e) => write!(f, "{e}"),
            Self::Einsum(e) => write!(f, "{e}"),
            Self::Check(e) => write!(f, "{e}"),
            Self::Codegen(e) => write!(f, "{e}"),
            Self::Format(file, e) => write!(f, "{file}: {e}"),
            Self::Tool(text) => write!(f, "{text}"),
        }
    }
}

impl std::error::Error for CliError {}

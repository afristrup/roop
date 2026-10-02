use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Io(String, std::io::Error),
    Config(roop_config::ConfigError),
    Parse(roop_syntax::ParseError),
    Check(roop_check::CheckError),
    Codegen(roop_llvm::CodegenError),
    Tool(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Usage(text) => write!(f, "{text}"),
            Self::Io(what, e) => write!(f, "{what}: {e}"),
            Self::Config(e) => write!(f, "{e}"),
            Self::Parse(e) => write!(f, "syntax error: {e}"),
            Self::Check(e) => write!(f, "{e}"),
            Self::Codegen(e) => write!(f, "{e}"),
            Self::Tool(text) => write!(f, "{text}"),
        }
    }
}

impl std::error::Error for CliError {}

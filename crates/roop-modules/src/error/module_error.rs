use roop_syntax::ParseError;
use std::fmt;

#[derive(Debug)]
pub enum ModuleError {
    Read(String, std::io::Error),
    Parse(String, ParseError),
    MissingFile { module: String, tried: String },
    UnknownModule(String),
    UnknownItem { module: String, name: String },
    Private { module: String, name: String },
    Duplicate { module: String, name: String },
    EmptyUse(String),
    NoParent(String),
    ImportCycle(String),
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Read(path, e) => write!(f, "cannot read {path}: {e}"),
            Self::Parse(path, e) => write!(f, "{path}: syntax error: {e}"),
            Self::MissingFile { module, tried } => {
                write!(f, "module `{module}` has no file (looked for {tried})")
            }
            Self::UnknownModule(name) => write!(f, "unknown module `{name}`"),
            Self::UnknownItem { module, name } => {
                write!(f, "module `{module}` has no item `{name}`")
            }
            Self::Private { module, name } => {
                write!(
                    f,
                    "`{name}` is private to module `{module}` (mark it `pub`)"
                )
            }
            Self::Duplicate { module, name } => {
                write!(
                    f,
                    "`{name}` is defined or imported twice in module `{module}`"
                )
            }
            Self::EmptyUse(module) => write!(f, "`use` needs a module and an item in `{module}`"),
            Self::NoParent(module) => write!(f, "module `{module}` has no parent for `super`"),
            Self::ImportCycle(module) => write!(f, "imports of `{module}` lead back to themselves"),
        }
    }
}

impl std::error::Error for ModuleError {}

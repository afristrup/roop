use crate::{CliError, find_tool};
use std::process::Command;

pub fn clang() -> Result<Command, CliError> {
    let path = find_tool("clang").ok_or_else(|| CliError::Tool("clang not found".into()))?;
    Ok(Command::new(path))
}

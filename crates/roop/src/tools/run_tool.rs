use crate::CliError;
use std::process::Command;

pub fn run_tool(mut command: Command) -> Result<Vec<u8>, CliError> {
    let name = command.get_program().to_string_lossy().into_owned();
    let output = command
        .output()
        .map_err(|e| CliError::Io(format!("cannot run {name}"), e))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(CliError::Tool(format!("{name} failed:\n{stdout}{stderr}")))
    }
}

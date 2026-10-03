use crate::{CliError, WeaveArgs};
use roop_weave::{emit_model, parse_model};

/// `roop weave`: compiles a model of reversible layers, as JSON, to roop code.
pub fn run_weave(args: &WeaveArgs) -> Result<(), CliError> {
    let name = args.input.display().to_string();
    let text = std::fs::read_to_string(&args.input).map_err(|e| CliError::Io(name.clone(), e))?;
    let model = parse_model(&text).map_err(|e| CliError::Tool(format!("{name}: {e}")))?;
    let code = emit_model(&model, args.tests, args.batch);
    match &args.output {
        Some(path) => {
            std::fs::write(path, code).map_err(|e| CliError::Io(path.display().to_string(), e))
        }
        None => {
            print!("{code}");
            Ok(())
        }
    }
}

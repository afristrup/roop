use crate::{CliError, FmtArgs, find_roop_files};
use roop_fmt::format_source;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// `roop fmt`: rewrites files in place, or with `--check` only reports them.
pub fn run_fmt(args: &FmtArgs) -> Result<(), CliError> {
    let start = args
        .paths
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));
    let config = crate::config_for(&start.join("x"))?;
    let mut format = config.format.clone();
    if let Some(width) = args.width {
        format.max_width = width;
    }
    if args.stdin {
        let mut src = String::new();
        std::io::stdin()
            .read_to_string(&mut src)
            .map_err(|e| CliError::Io("stdin".into(), e))?;
        let out = format_source(&src, &format).map_err(|e| CliError::Format("stdin".into(), e))?;
        std::io::stdout()
            .write_all(out.as_bytes())
            .map_err(|e| CliError::Io("stdout".into(), e))?;
        return Ok(());
    }
    let roots: Vec<&Path> = match args.paths.is_empty() {
        true => vec![config.root.as_path()],
        false => args.paths.iter().map(PathBuf::as_path).collect(),
    };
    let mut changed = 0;
    for root in roots {
        for file in find_roop_files(root)? {
            let name = file.display().to_string();
            let src = std::fs::read_to_string(&file).map_err(|e| CliError::Io(name.clone(), e))?;
            let out =
                format_source(&src, &format).map_err(|e| CliError::Format(name.clone(), e))?;
            if out == src {
                continue;
            }
            changed += 1;
            println!("{name}");
            if !args.check {
                std::fs::write(&file, out).map_err(|e| CliError::Io(name, e))?;
            }
        }
    }
    match (args.check, changed) {
        (true, n) if n > 0 => Err(CliError::Tool(format!("{n} files are not formatted"))),
        _ => Ok(()),
    }
}

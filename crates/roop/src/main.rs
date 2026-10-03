mod build;
mod cli;
mod error;
mod lsp;
mod tools;

use build::*;
use cli::*;
use error::*;
use lsp::*;
use tools::*;

/// The parser and the checks recurse once per nesting level, so deeply nested
/// programs need more than the main thread's stack.
const STACK_BYTES: usize = 512 * 1024 * 1024;

fn run() -> Result<(), CliError> {
    match parse_args(std::env::args().skip(1))? {
        Command::Build(args) => build(&args),
        Command::Lean(args) => run_lean(&args),
        Command::Fmt(args) => run_fmt(&args),
        Command::Test(args) => run_test(&args),
        Command::Run(args) => run_run(&args),
        Command::Weave(args) => run_weave(&args),
        Command::Lsp => serve(std::io::stdin().lock(), std::io::stdout().lock())
            .map_err(|e| CliError::Io("the language server".into(), e)),
    }
}

fn main() {
    let worker = std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(run)
        .expect("cannot start the compiler thread");
    let result = worker.join().expect("the compiler thread panicked");
    if let Err(error) = result {
        eprintln!("roop: {error}");
        std::process::exit(1);
    }
}

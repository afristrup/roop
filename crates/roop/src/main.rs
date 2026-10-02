mod build;
mod cli;
mod error;
mod tools;

use build::*;
use cli::*;
use error::*;
use tools::*;

fn main() {
    let result = parse_args(std::env::args().skip(1)).and_then(|command| match command {
        Command::Build(args) => build(&args),
        Command::Lean(args) => run_lean(&args),
    });
    if let Err(error) = result {
        eprintln!("roop: {error}");
        std::process::exit(1);
    }
}

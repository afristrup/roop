mod build;
mod cli;
mod error;
mod tools;

use build::*;
use cli::*;
use error::*;
use tools::*;

fn main() {
    let result = parse_args(std::env::args().skip(1)).and_then(|args| build(&args));
    if let Err(error) = result {
        eprintln!("roop: {error}");
        std::process::exit(1);
    }
}

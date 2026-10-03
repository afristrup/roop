mod fusion;
mod generics;
mod rename;
mod tests;

use fusion::*;
use rename::*;
pub use tests::{split_test, stage_name, strip_tests};

pub use fusion::fuse_parallel;
use generics::*;
pub use generics::{GenericError, monomorphize};

mod fusion;
mod generics;
mod rename;

use fusion::*;
use rename::*;

pub use fusion::fuse_parallel;
use generics::*;
pub use generics::{GenericError, monomorphize};

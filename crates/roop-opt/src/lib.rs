mod auto;
mod bennett;
mod einsum;
mod fusion;
mod generics;
mod rename;
mod tests;

use auto::*;
use bennett::*;
use einsum::*;
use fusion::*;
use rename::*;
pub use tests::{split_test, stage_name, strip_tests};

pub use auto::{AutoError, expand_auto};
pub use bennett::{BennettError, expand_bennett};
pub use einsum::{EinsumError, expand_einsum};
pub use fusion::fuse_parallel;
use generics::*;
pub use generics::{GenericError, infer_lengths, monomorphize};

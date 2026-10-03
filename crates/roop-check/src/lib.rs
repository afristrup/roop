mod driver;
mod error;
mod exprs;
mod places;
mod rules;
mod stmts;

pub use driver::check;
pub use error::CheckError;
pub use places::place_root;
pub use rules::check_parallel_body;
pub use rules::{TaskItem, task_groups};
pub use rules::{compliant, dual, unfold, well_formed};
pub use stmts::{
    BodyEffects, BodyFeatures, Mutability, body_effects, body_features, calls_in, channels_used,
    contains_irrev, fn_mutability, irreversible_fns, is_irreversible_fn, non_atomic_fns,
    precise_writes,
};

use driver::*;
use exprs::*;
use places::*;
use rules::*;
use stmts::*;

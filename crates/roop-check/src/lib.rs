mod driver;
mod error;
mod exprs;
mod places;
mod rules;
mod stmts;

pub use driver::check;
pub use error::CheckError;
pub use rules::check_parallel_body;
pub use rules::{TaskItem, task_groups};
pub use stmts::{
    BodyEffects, BodyFeatures, body_effects, body_features, channels_used, contains_irrev,
    irreversible_fns, is_irreversible_fn,
};

use driver::*;
use exprs::*;
use places::*;
use rules::*;
use stmts::*;

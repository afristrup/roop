mod driver;
mod error;
mod exprs;
mod places;
mod rules;
mod stmts;

pub use driver::check;
pub use error::CheckError;
pub use rules::check_parallel_body;

use driver::*;
use exprs::*;
use places::*;
use rules::*;
use stmts::*;

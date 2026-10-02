mod driver;
mod error;
mod exprs;
mod places;
mod rules;
mod stmts;

pub use driver::check;
pub use error::CheckError;

use driver::*;
use exprs::*;
use places::*;
use rules::*;
use stmts::*;

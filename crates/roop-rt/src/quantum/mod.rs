//! A state-vector simulator behind the `qubit` type: what `std::quantum`
//! calls. Gates are world operations with exact inverses; measurement is not.

mod amp;
mod apply_one;
mod device;
mod gate;
mod gates;
mod phase;
mod position;
mod roop_q_alloc;
mod roop_q_cx;
mod roop_q_free;
mod roop_q_measure;
mod roop_q_r;
mod roop_q_r_inv;
mod rotation;

pub use amp::*;
pub use apply_one::*;
pub use device::*;
pub use gate::*;
pub use gates::*;
pub use phase::*;
pub use position::*;
pub use roop_q_alloc::*;
pub use roop_q_cx::*;
pub use roop_q_free::*;
pub use roop_q_measure::*;
pub use roop_q_r::*;
pub use roop_q_r_inv::*;
pub use rotation::*;

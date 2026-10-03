//! A compiler from a model of reversible layers to roop code that runs it
//! forward and backward with weave, and to the tests that check that code
//! against a reference in doubles.

mod activation;
mod decl;
mod emit_backward;
mod emit_forward;
mod emit_grad;
mod emit_load;
mod emit_model;
mod emit_step;
mod emit_tests;
mod emit_train;
mod gradients;
mod layer;
mod layer_call;
mod loss;
mod model;
mod names;
mod param_list;
mod parse_model;
mod quantize;
mod reference;
mod tensor;
mod weave_error;

pub use activation::*;
pub use decl::*;
pub use emit_backward::*;
pub use emit_forward::*;
pub use emit_grad::*;
pub use emit_load::*;
pub use emit_model::*;
pub use emit_step::*;
pub use emit_tests::*;
pub use emit_train::*;
pub use gradients::*;
pub use layer::*;
pub use layer_call::*;
pub use loss::*;
pub use model::*;
pub use names::*;
pub use param_list::*;
pub use parse_model::*;
pub use quantize::*;
pub use reference::*;
pub use tensor::*;
pub use weave_error::*;

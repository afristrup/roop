use crate::{Activation, Tensor, spectral_norm};

/// The largest bound on the Lipschitz constant of a residual function that weave accepts.
pub const CONTRACTION_LIMIT: f64 = 0.9;

/// An upper bound on the Lipschitz constant of `W2 f(W1 x + b1) + b2`: the product
/// of the spectral norms of the weights and the slope of the activation.
pub fn contraction_bound(act: Activation, w1: &Tensor, w2: &Tensor) -> f64 {
    spectral_norm(w1) * spectral_norm(w2) * act.lipschitz()
}

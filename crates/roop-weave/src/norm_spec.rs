use crate::Tensor;

/// An RMS normalization of the hidden layer of a perceptron:
/// `gain * z / sqrt(mean(z^2) + eps)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Norm {
    pub eps: f64,
    pub gain: Tensor,
}

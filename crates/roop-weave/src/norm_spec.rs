use crate::Tensor;

/// A normalization of the hidden layer of a perceptron. With no `bias` it is the
/// RMS norm, `gain * z / sqrt(mean(z^2) + eps)`. With one it is a layer norm,
/// `gain * (z - mean(z)) / sqrt(var(z) + eps) + bias`.
#[derive(Clone, Debug, PartialEq)]
pub struct Norm {
    pub eps: f64,
    pub gain: Tensor,
    pub bias: Option<Tensor>,
}

impl Norm {
    /// The gain, then the bias if there is one.
    pub fn tensors(&self) -> Vec<&Tensor> {
        std::iter::once(&self.gain).chain(&self.bias).collect()
    }

    pub fn tensors_mut(&mut self) -> Vec<&mut Tensor> {
        std::iter::once(&mut self.gain)
            .chain(&mut self.bias)
            .collect()
    }
}

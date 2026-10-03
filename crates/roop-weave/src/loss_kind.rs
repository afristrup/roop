/// What the output is scored against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossKind {
    /// |q - t|^2 / 2
    Mse,
    /// The cross entropy of sigmoid(q) against targets between 0 and 1.
    Sigmoid,
    /// The cross entropy of softmax(q) against targets that sum to 1.
    Softmax,
}

impl LossKind {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "mse" => Some(Self::Mse),
            "sigmoid" => Some(Self::Sigmoid),
            "softmax" => Some(Self::Softmax),
            _ => None,
        }
    }
}

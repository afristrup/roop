use crate::{Activation, Tensor};

/// One reversible layer of the network.
#[derive(Clone, Debug, PartialEq)]
pub enum Layer {
    /// A leapfrog step with `W^T f(W q + b)` as its force.
    Leapfrog {
        act: Activation,
        w: Tensor,
        b: Tensor,
    },
    /// Half of a coupling: one half of the state takes in a perceptron of the other.
    Mlp {
        act: Activation,
        w1: Tensor,
        b1: Tensor,
        w2: Tensor,
        b2: Tensor,
    },
}

impl Layer {
    pub fn tensors(&self) -> Vec<&Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
        }
    }

    pub fn tensors_mut(&mut self) -> Vec<&mut Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
        }
    }

    pub fn activation(&self) -> Activation {
        match self {
            Self::Leapfrog { act, .. } | Self::Mlp { act, .. } => *act,
        }
    }

    /// The width of the hidden layer.
    pub fn hidden(&self) -> usize {
        match self {
            Self::Leapfrog { w, .. } => w.dims[0],
            Self::Mlp { w1, .. } => w1.dims[0],
        }
    }
}

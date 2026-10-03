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
    /// Half of a coupling: one half of the state takes in linear attention over
    /// the other, read as `seq` rows.
    Attention {
        seq: usize,
        wq: Tensor,
        wk: Tensor,
        wv: Tensor,
    },
}

impl Layer {
    pub fn tensors(&self) -> Vec<&Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
            Self::Attention { wq, wk, wv, .. } => vec![wq, wk, wv],
        }
    }

    pub fn tensors_mut(&mut self) -> Vec<&mut Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
            Self::Attention { wq, wk, wv, .. } => vec![wq, wk, wv],
        }
    }

    /// The pointwise function, for the layers that have one.
    pub fn activation(&self) -> Option<Activation> {
        match self {
            Self::Leapfrog { act, .. } | Self::Mlp { act, .. } => Some(*act),
            Self::Attention { .. } => None,
        }
    }

    /// Whether the layer is one half of a coupling, so that they alternate.
    pub fn is_half_step(&self) -> bool {
        matches!(self, Self::Mlp { .. } | Self::Attention { .. })
    }

    /// The width of the hidden layer, or of a row for attention.
    pub fn hidden(&self) -> usize {
        match self {
            Self::Leapfrog { w, .. } => w.dims[0],
            Self::Mlp { w1, .. } => w1.dims[0],
            Self::Attention { wq, .. } => wq.dims[0],
        }
    }
}

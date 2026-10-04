use crate::{Activation, Norm, Tensor};

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
        norm: Option<Norm>,
    },
    /// An invertible residual block `x + F(x)` on q, with `F` a perceptron that is a
    /// contraction and `iters` cells in the chain that inverts it.
    Residual {
        act: Activation,
        w1: Tensor,
        b1: Tensor,
        w2: Tensor,
        b2: Tensor,
        iters: usize,
    },
    /// Half of a coupling: one half of the state takes in linear attention over
    /// the other, read as `seq` rows.
    Attention {
        seq: usize,
        wq: Tensor,
        wk: Tensor,
        wv: Tensor,
    },
    /// Half of a coupling: one half of the state takes in a one-dimensional
    /// convolution of the other, read as `channels` rows, with a kernel of
    /// `kernel` numbers, a weight of `channels` rows of `channels * kernel`.
    Conv {
        channels: usize,
        kernel: usize,
        w: Tensor,
        b: Tensor,
    },
}

impl Layer {
    pub fn tensors(&self) -> Vec<&Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp {
                w1,
                b1,
                w2,
                b2,
                norm,
                ..
            } => {
                let mut all = vec![w1, b1, w2, b2];
                all.extend(norm.iter().flat_map(Norm::tensors));
                all
            }
            Self::Residual { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
            Self::Attention { wq, wk, wv, .. } => vec![wq, wk, wv],
            Self::Conv { w, b, .. } => vec![w, b],
        }
    }

    pub fn tensors_mut(&mut self) -> Vec<&mut Tensor> {
        match self {
            Self::Leapfrog { w, b, .. } => vec![w, b],
            Self::Mlp {
                w1,
                b1,
                w2,
                b2,
                norm,
                ..
            } => {
                let mut all = vec![w1, b1, w2, b2];
                all.extend(norm.iter_mut().flat_map(Norm::tensors_mut));
                all
            }
            Self::Residual { w1, b1, w2, b2, .. } => vec![w1, b1, w2, b2],
            Self::Attention { wq, wk, wv, .. } => vec![wq, wk, wv],
            Self::Conv { w, b, .. } => vec![w, b],
        }
    }

    /// The pointwise function, for the layers that have one.
    pub fn activation(&self) -> Option<Activation> {
        match self {
            Self::Leapfrog { act, .. } | Self::Mlp { act, .. } | Self::Residual { act, .. } => {
                Some(*act)
            }
            Self::Attention { .. } | Self::Conv { .. } => None,
        }
    }

    /// Whether the layer is one half of a coupling, so that they alternate.
    pub fn is_half_step(&self) -> bool {
        matches!(
            self,
            Self::Mlp { .. } | Self::Attention { .. } | Self::Conv { .. }
        )
    }

    /// The width of the hidden layer, or of a row for attention.
    pub fn hidden(&self) -> usize {
        match self {
            Self::Leapfrog { w, .. } => w.dims[0],
            Self::Mlp { w1, .. } | Self::Residual { w1, .. } => w1.dims[0],
            Self::Attention { wq, .. } => wq.dims[0],
            Self::Conv { channels, .. } => *channels,
        }
    }
}

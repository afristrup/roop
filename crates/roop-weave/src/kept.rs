use crate::{Layer, Model};

/// A residual block whose weights the training step keeps under a bound.
pub struct Kept {
    pub n: usize,
    pub m: usize,
    pub w1: String,
    pub w2: String,
    pub slope: i64,
    pub cap: i64,
}

impl Kept {
    /// The blocks of the model that have `keep_contraction`, in layer order.
    pub fn of(model: &Model) -> Vec<Kept> {
        let kept = |layer: &Layer| match layer {
            Layer::Residual {
                act,
                w1,
                w2,
                keep: Some(keep),
                ..
            } => Some(Kept {
                n: w1.dims[1],
                m: w1.dims[0],
                w1: w1.name.clone(),
                w2: w2.name.clone(),
                slope: (act.lipschitz() * 4096.0).ceil() as i64,
                cap: (keep * 4096.0).floor() as i64,
            }),
            _ => None,
        };
        model.layers.iter().filter_map(kept).collect()
    }
}

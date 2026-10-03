use crate::{LossKind, Model};

/// A fixed input and target, on the grid, that the tests of a model use. The
/// target of a softmax is one-hot, since its gradient is `p - t` only for
/// targets that sum to 1.
pub fn sample(model: &Model) -> (Vec<f64>, Vec<f64>) {
    let x = (0..model.width)
        .map(|i| ((i * 3 + 1) % 7) as f64 / 4.0 - 0.75)
        .collect();
    let t = (0..model.outputs)
        .map(|k| match model.loss {
            LossKind::Mse => ((k * 2 + 1) % 3) as f64 / 4.0 - 0.25,
            LossKind::Sigmoid => (k % 2) as f64,
            LossKind::Softmax => (k == 0) as u8 as f64,
        })
        .collect();
    (x, t)
}

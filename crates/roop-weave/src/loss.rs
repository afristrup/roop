use crate::{Model, forward};

/// |q - t|^2 / 2 over the outputs.
pub fn loss(model: &Model, input: &[f64], target: &[f64]) -> f64 {
    let out = forward(model, input);
    out.iter()
        .zip(target)
        .map(|(q, t)| (q - t) * (q - t) / 2.0)
        .sum()
}

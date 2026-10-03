use crate::{LossKind, Model, forward};

fn logistic(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// The loss of the model on a sample, in doubles and with exact functions: half
/// the squared error, or a cross entropy.
pub fn loss(model: &Model, input: &[f64], target: &[f64]) -> f64 {
    let out = forward(model, input);
    match model.loss {
        LossKind::Mse => out
            .iter()
            .zip(target)
            .map(|(q, t)| (q - t) * (q - t) / 2.0)
            .sum(),
        LossKind::Sigmoid => out
            .iter()
            .zip(target)
            .map(|(q, t)| -t * logistic(*q).ln() - (1.0 - t) * logistic(-*q).ln())
            .sum(),
        LossKind::Softmax => {
            let top = out.iter().cloned().fold(f64::MIN, f64::max);
            let log_sum = out.iter().map(|q| (q - top).exp()).sum::<f64>().ln() + top;
            out.iter()
                .zip(target)
                .map(|(q, t)| -t * (q - log_sum))
                .sum()
        }
    }
}

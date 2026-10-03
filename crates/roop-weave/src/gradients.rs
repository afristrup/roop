use crate::{Model, loss};

/// The gradient of the loss with respect to every number of every tensor, by
/// central differences, in the order of `Model::tensors`.
pub fn gradients(model: &Model, input: &[f64], target: &[f64]) -> Vec<Vec<f64>> {
    const EPS: f64 = 1e-5;
    let mut probe = model.clone();
    let shape: Vec<usize> = model.tensors().iter().map(|t| t.data.len()).collect();
    let mut all = Vec::new();
    for (t, len) in shape.into_iter().enumerate() {
        let mut grad = Vec::with_capacity(len);
        for e in 0..len {
            let at = |probe: &mut Model, delta: f64| probe.tensors_mut()[t].data[e] += delta;
            at(&mut probe, EPS);
            let up = loss(&probe, input, target);
            at(&mut probe, -2.0 * EPS);
            let down = loss(&probe, input, target);
            at(&mut probe, EPS);
            grad.push((up - down) / (2.0 * EPS));
        }
        all.push(grad);
    }
    all
}

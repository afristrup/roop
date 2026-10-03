use crate::Model;

/// A fixed input and target, on the grid, that the tests of a model use.
pub fn sample(model: &Model) -> (Vec<f64>, Vec<f64>) {
    let x = (0..model.width)
        .map(|i| ((i * 3 + 1) % 7) as f64 / 4.0 - 0.75)
        .collect();
    let t = (0..model.outputs)
        .map(|k| ((k * 2 + 1) % 3) as f64 / 4.0 - 0.25)
        .collect();
    (x, t)
}

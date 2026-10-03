use crate::{Model, sample};

/// `rows` different inputs and targets, on the grid, for the tests of a batch:
/// the fixed sample with its numbers rotated by the row.
pub fn batch_sample(model: &Model, rows: usize) -> Vec<(Vec<f64>, Vec<f64>)> {
    let (x, t) = sample(model);
    let rotated = |v: &[f64], by: usize| (0..v.len()).map(|i| v[(i + by) % v.len()]).collect();
    (0..rows)
        .map(|row| (rotated(&x, row), rotated(&t, row)))
        .collect()
}

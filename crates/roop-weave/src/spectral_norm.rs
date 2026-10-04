use crate::Tensor;

/// The largest singular value of a matrix, by power iteration on `W^T W`, taken 1
/// percent high because the iteration comes up from below.
pub fn spectral_norm(w: &Tensor) -> f64 {
    let (rows, cols) = (w.dims[0], w.dims[1]);
    let mut v = vec![1.0 / (cols as f64).sqrt(); cols];
    let mut norm = 0.0;
    for _ in 0..500 {
        let wv: Vec<f64> = (0..rows)
            .map(|r| (0..cols).map(|c| w.at(r, c) * v[c]).sum())
            .collect();
        let next: Vec<f64> = (0..cols)
            .map(|c| (0..rows).map(|r| w.at(r, c) * wv[r]).sum())
            .collect();
        norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm == 0.0 {
            return 0.0;
        }
        v = next.iter().map(|x| x / norm).collect();
    }
    norm.sqrt() * 1.01
}

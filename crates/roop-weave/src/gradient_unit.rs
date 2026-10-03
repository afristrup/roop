use crate::Tensor;

/// How many Q12 units the gradient of a parameter is counted in: a matrix's
/// gradient is the sum of products of two Q12 numbers that are not divided by
/// 4096, so it is in Q24, and a vector's is in Q12.
pub fn gradient_unit(parameter: &Tensor) -> i64 {
    match parameter.dims.len() {
        2 => 4096,
        _ => 1,
    }
}

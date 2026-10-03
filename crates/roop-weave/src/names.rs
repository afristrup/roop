use crate::Tensor;

pub fn names(tensors: &[impl std::borrow::Borrow<Tensor>]) -> Vec<String> {
    tensors.iter().map(|t| t.borrow().name.clone()).collect()
}

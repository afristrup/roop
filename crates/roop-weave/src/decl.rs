use crate::Tensor;

/// `w0: &[[i64; 4]; 3]`, or `&mut` for a parameter that is changed.
pub fn decl(tensor: &Tensor, mutable: bool) -> String {
    let reference = if mutable { "&mut " } else { "&" };
    format!("{}: {reference}{}", tensor.name, tensor.roop_type())
}

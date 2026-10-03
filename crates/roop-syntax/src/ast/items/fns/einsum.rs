use crate::Type;

/// What an `einsum fn` contracts: the subscripts, as `"ij,jk->ik"`, and the
/// type of the numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Einsum {
    pub spec: String,
    pub elem: Type,
}

use crate::{EinsumError, build_sum, build_wide, parse_spec};
use roop_syntax::{FnDef, Type};

/// The functions for an `einsum fn`: `out` gets the sum, over the labels that
/// are not in the output, of the product of the elements of the operands `x0`,
/// `x1`, and so on. The sum is added to `out`, so a zero `out` ends up holding
/// the contraction, and running it backward takes it off again.
///
/// The element type says what a product is. In `i64` and `f64` it is the
/// product. In `q12` the numbers are fixed point, x * 4096, so each product is
/// divided by 4096. In `q12w` the products of a sum are added exactly and the
/// sum is divided by 4096 once, which is more exact and is one matrix product
/// that the compiler can give to the matrix unit; that needs the helper
/// `<name>__sum`, so there are two functions.
pub fn build_einsum(def: &FnDef) -> Result<Vec<FnDef>, EinsumError> {
    let einsum = def.einsum.as_ref().expect("an einsum fn");
    let name = &def.name;
    let Type::Named(elem) = &einsum.elem else {
        return Err(EinsumError::BadType(name.clone()));
    };
    if !matches!(elem.as_str(), "i64" | "f64" | "q12" | "q12w") {
        return Err(EinsumError::BadType(name.clone()));
    }
    let storage = match elem.starts_with("q12") {
        true => Type::Named("i64".into()),
        false => einsum.elem.clone(),
    };
    let spec = parse_spec(name, &einsum.spec)?;
    if elem == "q12w" && !spec.reduced().is_empty() {
        let sum = format!("{name}__sum");
        return Ok(vec![
            build_sum(&sum, false, &spec, &storage, false),
            build_wide(name, &sum, def.public, &spec, &storage),
        ]);
    }
    let scaled = elem.starts_with("q12");
    Ok(vec![build_sum(name, def.public, &spec, &storage, scaled)])
}

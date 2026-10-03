use crate::{Spec, operand_type};
use roop_syntax::{Param, Type};

/// `out` and the operands `x0`, `x1`, and so on, as the arrays the spec says,
/// with `out` the only one that is written.
pub fn einsum_params(spec: &Spec, storage: &Type) -> Vec<Param> {
    let by_ref = |ty: Type, mutable: bool| Type::Ref {
        mutable,
        inner: Box::new(ty),
    };
    let mut params = vec![Param {
        name: "out".into(),
        ty: by_ref(operand_type(storage, &spec.output), true),
    }];
    for (k, labels) in spec.inputs.iter().enumerate() {
        params.push(Param {
            name: format!("x{k}"),
            ty: by_ref(operand_type(storage, labels), false),
        });
    }
    params
}

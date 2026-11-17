use crate::{CheckError, QubitScope, check_qubit_block, holds_qubit};
use roop_syntax::{Item, Program, Type};

/// The rules of the `qubit` type: a function takes a qubit by `&mut`, since
/// a gate changes its qubits and the device cannot hand out a copy; a struct
/// does not hold one; and a body never reads one as a value.
pub fn check_qubits(program: &Program) -> Result<(), CheckError> {
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                let mut scope = QubitScope::default();
                for p in &f.params {
                    let qubit = holds_qubit(&p.ty);
                    if qubit && !matches!(p.ty, Type::Ref { mutable: true, .. }) {
                        return Err(CheckError::QubitParam {
                            function: f.name.clone(),
                            param: p.name.clone(),
                            span: f.body.span,
                        });
                    }
                    scope.push(&p.name, qubit);
                }
                check_qubit_block(&f.body, &mut scope)?;
            }
            Item::Struct(def) => {
                if let Some(field) = def.fields.iter().find(|field| holds_qubit(&field.ty)) {
                    return Err(CheckError::QubitInStruct {
                        name: def.name.clone(),
                        field: field.name.clone(),
                        span: def.span,
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

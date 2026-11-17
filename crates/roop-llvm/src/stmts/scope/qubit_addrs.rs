use crate::{CodegenError, FnGen, llvm_type};
use roop_syntax::Type;

/// The address of every qubit in a value of type `ty` at `addr`, in order:
/// one for a `qubit`, each element of an array of them, none otherwise.
pub fn qubit_addrs(g: &mut FnGen, ty: &Type, addr: &str) -> Result<Vec<String>, CodegenError> {
    match ty {
        Type::Named(name) if name == "qubit" => Ok(vec![addr.into()]),
        Type::Array(elem, len) => {
            let array = llvm_type(g.ctx, ty)?;
            let mut out = Vec::new();
            for i in 0..*len {
                let at = format!("%{}", g.fresh("q"));
                g.emit(&format!(
                    "{at} = getelementptr inbounds {array}, ptr {addr}, i64 0, i64 {i}"
                ));
                out.extend(qubit_addrs(g, elem, &at)?);
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

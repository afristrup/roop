use crate::{Binding, HybridError, Int, Lowerer, Value, roop_expr, unsupported};
use roop_syntax::{Place, Span};

/// The binding of a place: a name, or an element of a qubit array at an
/// index known when compiling, since a circuit names its qubits.
pub fn roop_place(l: &Lowerer, place: &Place, span: Span) -> Result<Binding, HybridError> {
    match place {
        Place::Var(name) => l.lookup(name, span),
        Place::Index(base, index) => {
            let Binding::Qubits(qubits) = roop_place(l, base, span)? else {
                return Err(unsupported("an array that is not of qubits", span));
            };
            let Value::Int(Int::Lit(i)) = roop_expr(l, index, span)? else {
                return Err(unsupported("a qubit index not known when compiling", span));
            };
            let q = usize::try_from(i).ok().and_then(|i| qubits.get(i).copied());
            q.map(Binding::Qubit)
                .ok_or_else(|| unsupported("an index out of bounds", span))
        }
        Place::Field(..) => Err(unsupported("a struct field", span)),
    }
}

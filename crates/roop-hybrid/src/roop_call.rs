use crate::{
    Binding, HybridError, Int, Lowerer, MAX_INLINE_DEPTH, Op, Value, push_gate, roop_block,
    roop_expr, roop_gate, roop_place, unsupported,
};
use roop_syntax::{Expr, Span, Type};

/// A call, inlined: a gate of the device, a measurement, or a roop function
/// with its parameters bound to the arguments. `inverse` runs it backward.
/// Calls into `std::io` are left out, since a circuit has no output but its
/// bits.
pub fn roop_call(
    l: &mut Lowerer,
    callee: &str,
    args: &[Expr],
    inverse: bool,
    span: Span,
    out: &mut Vec<Op>,
) -> Result<(), HybridError> {
    if callee.starts_with("std__io__") {
        return Ok(());
    }
    let bindings: Vec<Binding> = args
        .iter()
        .map(|a| match a {
            Expr::Place(p) => roop_place(l, p, span),
            e => Ok(match roop_expr(l, e, span)? {
                Value::Int(i) => Binding::IntValue(i),
                Value::Bit(c) => Binding::BitValue(c),
            }),
        })
        .collect::<Result<_, _>>()?;
    let qubits: Vec<usize> = bindings
        .iter()
        .filter_map(|b| {
            if let Binding::Qubit(q) = b {
                Some(*q)
            } else {
                None
            }
        })
        .collect();
    let constant = bindings.iter().find_map(|b| match b {
        Binding::IntValue(Int::Lit(k)) => Some(*k),
        _ => None,
    });
    if callee == "roop_q_measure" {
        return match (inverse, &bindings[..]) {
            (false, [Binding::Qubit(qubit), Binding::Bit(bit)]) => {
                out.push(Op::Measure {
                    bit: *bit,
                    qubit: *qubit,
                });
                Ok(())
            }
            _ => Err(unsupported("this measurement", span)),
        };
    }
    if callee.starts_with("roop_q_") {
        let gates = roop_gate(callee, &qubits, constant, inverse)
            .ok_or_else(|| unsupported(&format!("`{callee}`"), span))?;
        gates.into_iter().for_each(|g| push_gate(out, g));
        return Ok(());
    }
    let def = *l
        .fns
        .get(callee)
        .ok_or_else(|| unsupported(&format!("`{callee}`"), span))?;
    if def.external {
        return Err(unsupported(&format!("the extern `{callee}`"), span));
    }
    if inverse && l.irreversible.contains(callee) {
        return Err(unsupported(
            &format!("running the irreversible `{callee}` backward"),
            span,
        ));
    }
    if l.depth == MAX_INLINE_DEPTH {
        return Err(unsupported(&format!("recursion into `{callee}`"), span));
    }
    for (param, binding) in def.params.iter().zip(&bindings) {
        let scalar =
            matches!(&param.ty, Type::Ref { inner, .. } if matches!(**inner, Type::Named(_)));
        if !scalar && !matches!(binding, Binding::Qubits(_)) {
            return Err(unsupported(
                &format!("the parameter `{}` of `{callee}`", param.name),
                span,
            ));
        }
    }
    let saved = std::mem::replace(
        &mut l.frame,
        def.params
            .iter()
            .map(|p| p.name.clone())
            .zip(bindings)
            .collect(),
    );
    l.depth += 1;
    let result = roop_block(l, &def.body, inverse, out);
    l.depth -= 1;
    l.frame = saved;
    result
}

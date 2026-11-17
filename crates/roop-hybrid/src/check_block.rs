use crate::{HybridError, Kind, Signature, Stmt, check_expr, check_gate, lookup};

/// The judgment `s1 ⊢ p : s2` of HQbricks∞: extends `sig` from s1 to s2, or
/// fails on a use of memory that is not allocated, a second allocation, `if`
/// branches that allocate differently, or an allocation in a loop.
pub fn check_block(stmts: &[Stmt], sig: &mut Signature) -> Result<(), HybridError> {
    for stmt in stmts {
        match stmt {
            Stmt::Skip => {}
            Stmt::Declare { kind, name, line } => {
                if sig.get(name).is_some() {
                    return Err(HybridError::Redeclared {
                        line: *line,
                        name: name.clone(),
                    });
                }
                sig.entries.push((name.clone(), *kind));
            }
            Stmt::Gate { gate, qubits, line } => check_gate(*gate, qubits, sig, *line)?,
            Stmt::Assign { name, value, line } => match sig.get(name) {
                Some(kind @ (Kind::Bit | Kind::Int)) => check_expr(value, kind, sig, *line)?,
                _ => lookup(sig, name, Kind::Int, *line)?,
            },
            Stmt::Measure { bit, qubit, line } => {
                lookup(sig, bit, Kind::Bit, *line)?;
                lookup(sig, qubit, Kind::Qubit, *line)?;
            }
            Stmt::If {
                cond,
                then,
                els,
                line,
            } => {
                check_expr(cond, Kind::Bit, sig, *line)?;
                let (mut a, mut b) = (sig.clone(), sig.clone());
                check_block(then, &mut a)?;
                check_block(els, &mut b)?;
                let names: Vec<String> = a
                    .names()
                    .filter(|n| b.get(n).is_none())
                    .chain(b.names().filter(|n| a.get(n).is_none()))
                    .map(String::from)
                    .collect();
                if !names.is_empty() {
                    return Err(HybridError::Branches { line: *line, names });
                }
                *sig = a;
            }
            Stmt::While { cond, body, line } => {
                check_expr(cond, Kind::Bit, sig, *line)?;
                let mut inner = sig.clone();
                check_block(body, &mut inner)?;
                if let Some((name, _)) = inner.entries.get(sig.entries.len()) {
                    return Err(HybridError::LoopAllocation {
                        line: *line,
                        name: name.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

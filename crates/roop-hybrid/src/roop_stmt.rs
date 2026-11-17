use crate::{
    Binding, Cond, Gate, HybridError, Int, Lowerer, Op, Value, push_gate, roop_block, roop_call,
    roop_expr, roop_place, roop_update, unsupported,
};
use roop_syntax::{Expr, OverwriteOp, Span, Stmt, StmtKind, Type};

/// One statement, or its inverse when `backward`. An `if` runs backward on
/// its exit assertion and a loop from its `until`. The assertions themselves
/// are roop's run-time checks and do not reach the circuit. A kept value
/// goes to a `_kept` register, since the world holds it.
pub fn roop_stmt(
    l: &mut Lowerer,
    stmt: &Stmt,
    backward: bool,
    out: &mut Vec<Op>,
) -> Result<(), HybridError> {
    let span = stmt.span;
    let cond = |l: &Lowerer, e: &Expr| match roop_expr(l, e, span)? {
        Value::Bit(c) => Ok(c),
        Value::Int(_) => Err(unsupported("an int as a condition", span)),
    };
    match &stmt.kind {
        StmtKind::Update { target, op, value } => {
            out.push(roop_update(l, target, *op, value, backward, span)?)
        }
        StmtKind::Swap(a, b) => swap(l, a, b, span, out)?,
        StmtKind::If {
            cond: c,
            then_block,
            else_block,
            exit,
        } => {
            let test = cond(l, if backward { exit } else { c })?;
            let (mut then, mut els) = (Vec::new(), Vec::new());
            roop_block(l, then_block, backward, &mut then)?;
            roop_block(l, else_block, backward, &mut els)?;
            out.push(Op::If {
                cond: test,
                then,
                els,
            });
        }
        StmtKind::From {
            entry,
            body,
            step,
            until,
        } => {
            let stop = cond(l, if backward { entry } else { until })?;
            roop_block(l, body, backward, out)?;
            let mut round = Vec::new();
            roop_block(l, step, backward, &mut round)?;
            roop_block(l, body, backward, &mut round)?;
            out.push(Op::While {
                cond: Cond::Not(Box::new(stop)),
                body: round,
            });
        }
        StmtKind::Ancilla {
            name,
            ty,
            init,
            body,
        } => {
            let binding = allocate(l, name, ty, span)?;
            match (&binding, roop_expr(l, init, span)) {
                (Binding::Int(reg), Ok(Value::Int(value))) => {
                    out.push(Op::SetInt { reg: *reg, value })
                }
                (Binding::Bit(bit), Ok(Value::Bit(value))) => {
                    out.push(Op::SetBit { bit: *bit, value })
                }
                (Binding::Qubit(_) | Binding::Qubits(_), _) => {}
                _ => return Err(unsupported("this ancilla start", span)),
            }
            l.frame.push((name.clone(), binding));
            roop_block(l, body, backward, out)?;
            l.frame.pop();
        }
        StmtKind::Call { callee, args, .. } => roop_call(l, callee, args, backward, span, out)?,
        StmtKind::Uncall { callee, args, .. } => roop_call(l, callee, args, !backward, span, out)?,
        StmtKind::Irrev(body) if !backward => roop_block(l, body, false, out)?,
        StmtKind::Block(body) => roop_block(l, body, backward, out)?,
        StmtKind::Overwrite { target, op, value } if !backward => {
            let v = roop_expr(l, value, span)?;
            out.push(match (roop_place(l, target, span)?, op, v) {
                (Binding::Int(reg), OverwriteOp::Assign, Value::Int(value)) => {
                    Op::SetInt { reg, value }
                }
                (Binding::Int(reg), OverwriteOp::Rem, Value::Int(v)) => Op::SetInt {
                    reg,
                    value: Int::Rem(Box::new(Int::Reg(reg)), Box::new(v)),
                },
                (Binding::Bit(bit), OverwriteOp::Assign, Value::Bit(value)) => {
                    Op::SetBit { bit, value }
                }
                _ => return Err(unsupported("this assignment", span)),
            });
        }
        StmtKind::Keep(place) if !backward => match roop_place(l, place, span)? {
            Binding::Int(reg) => {
                let name = format!("{}_kept", l.layout.ints[reg]);
                let kept = Lowerer::fresh(&mut l.layout.ints, &name);
                out.push(Op::SetInt {
                    reg: kept,
                    value: Int::Reg(reg),
                });
                out.push(Op::SetInt {
                    reg,
                    value: Int::Lit(0),
                });
            }
            Binding::Bit(bit) => {
                let name = format!("{}_kept", l.layout.bits[bit]);
                let kept = Lowerer::fresh(&mut l.layout.bits, &name);
                out.push(Op::SetBit {
                    bit: kept,
                    value: Cond::Bit(bit),
                });
                out.push(Op::SetBit {
                    bit,
                    value: Cond::Lit(false),
                });
            }
            _ => return Err(unsupported("keeping this", span)),
        },
        StmtKind::Irrev(_) | StmtKind::Overwrite { .. } | StmtKind::Keep(_) => {
            return Err(unsupported("running irreversible code backward", span));
        }
        other => return Err(unsupported(kind_name(other), span)),
    }
    Ok(())
}

/// A fresh register or qubits for an ancilla. A qubit ancilla's block ends
/// with it back in |0⟩, which roop checks, so a loop can reuse it.
fn allocate(l: &mut Lowerer, name: &str, ty: &Type, span: Span) -> Result<Binding, HybridError> {
    Ok(match ty {
        Type::Named(t) if t == "qubit" => {
            Binding::Qubit(Lowerer::fresh(&mut l.layout.qubits, name))
        }
        Type::Named(t) if t == "i64" => Binding::Int(Lowerer::fresh(&mut l.layout.ints, name)),
        Type::Named(t) if t == "bool" => Binding::Bit(Lowerer::fresh(&mut l.layout.bits, name)),
        Type::Array(elem, len) if matches!(&**elem, Type::Named(t) if t == "qubit") => {
            Binding::Qubits(
                (0..*len)
                    .map(|i| Lowerer::fresh(&mut l.layout.qubits, &format!("{name}{i}")))
                    .collect(),
            )
        }
        _ => {
            return Err(unsupported(
                &format!("the ancilla `{name}` of this type"),
                span,
            ));
        }
    })
}

/// `a <=> b`: three CNOTs for qubits, three xors for ints and bools.
fn swap(
    l: &Lowerer,
    a: &roop_syntax::Place,
    b: &roop_syntax::Place,
    span: Span,
    out: &mut Vec<Op>,
) -> Result<(), HybridError> {
    match (roop_place(l, a, span)?, roop_place(l, b, span)?) {
        (Binding::Qubit(p), Binding::Qubit(q)) => {
            for g in [Gate::Cnot(p, q), Gate::Cnot(q, p), Gate::Cnot(p, q)] {
                push_gate(out, g);
            }
        }
        (Binding::Int(p), Binding::Int(q)) => {
            let xor = |x: usize, y: usize| Op::SetInt {
                reg: x,
                value: Int::Xor(Box::new(Int::Reg(x)), Box::new(Int::Reg(y))),
            };
            out.extend([xor(p, q), xor(q, p), xor(p, q)]);
        }
        (Binding::Bit(p), Binding::Bit(q)) => {
            let xor = |x: usize, y: usize| Op::SetBit {
                bit: x,
                value: Cond::Xor(Box::new(Cond::Bit(x)), Box::new(Cond::Bit(y))),
            };
            out.extend([xor(p, q), xor(q, p), xor(p, q)]);
        }
        _ => return Err(unsupported("this swap", span)),
    }
    Ok(())
}

fn kind_name(kind: &StmtKind) -> &'static str {
    match kind {
        StmtKind::Match { .. } => "`match`",
        StmtKind::Borrow { .. } => "a borrow",
        StmtKind::Try { .. } => "`try`",
        StmtKind::Logged { .. } => "`logged`",
        StmtKind::Push { .. } | StmtKind::Pop { .. } => "a stack",
        StmtKind::Chan { .. } | StmtKind::Send { .. } | StmtKind::Recv { .. } => "a channel",
        _ => "this statement",
    }
}

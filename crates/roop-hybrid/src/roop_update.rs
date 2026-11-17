use crate::{
    Binding, Cond, HybridError, Int, Lowerer, Op, Value, roop_expr, roop_place, unsupported,
};
use roop_syntax::{Expr, Place, Span, UpdateOp};

/// `target op= value`, or its inverse.
pub fn roop_update(
    l: &Lowerer,
    target: &Place,
    op: UpdateOp,
    value: &Expr,
    backward: bool,
    span: Span,
) -> Result<Op, HybridError> {
    let op = if backward { op.inverse() } else { op };
    let value = roop_expr(l, value, span)?;
    match (roop_place(l, target, span)?, value) {
        (Binding::Int(reg), Value::Int(v)) => {
            let (old, v) = (Box::new(Int::Reg(reg)), Box::new(v));
            let value = match op {
                UpdateOp::Add => Int::Add(old, v),
                UpdateOp::Sub => Int::Sub(old, v),
                UpdateOp::Xor => Int::Xor(old, v),
                UpdateOp::Mul => Int::Mul(old, v),
                UpdateOp::Div => Int::Div(old, v),
            };
            Ok(Op::SetInt { reg, value })
        }
        (Binding::Bit(bit), Value::Bit(v)) if op == UpdateOp::Xor => Ok(Op::SetBit {
            bit,
            value: Cond::Xor(Box::new(Cond::Bit(bit)), Box::new(v)),
        }),
        _ => Err(unsupported("this update", span)),
    }
}

use crate::{Binding, Cond, HybridError, Int, Lowerer, Value, roop_place, unsupported};
use roop_syntax::{BinOp, Expr, Span, UnOp};

/// A roop expression over ints and bools as a hybrid value.
pub fn roop_expr(l: &Lowerer, expr: &Expr, span: Span) -> Result<Value, HybridError> {
    let int = |e: &Expr| match roop_expr(l, e, span)? {
        Value::Int(i) => Ok(Box::new(i)),
        Value::Bit(_) => Err(unsupported("a bool where an int belongs", span)),
    };
    let bit = |e: &Expr| match roop_expr(l, e, span)? {
        Value::Bit(c) => Ok(Box::new(c)),
        Value::Int(_) => Err(unsupported("an int where a bool belongs", span)),
    };
    Ok(match expr {
        Expr::Int(n) => Value::Int(Int::Lit(*n)),
        Expr::Bool(b) => Value::Bit(Cond::Lit(*b)),
        Expr::Place(place) => match roop_place(l, place, span)? {
            Binding::Int(r) => Value::Int(Int::Reg(r)),
            Binding::Bit(b) => Value::Bit(Cond::Bit(b)),
            Binding::IntValue(i) => Value::Int(i),
            Binding::BitValue(c) => Value::Bit(c),
            Binding::Qubit(_) | Binding::Qubits(_) => {
                return Err(unsupported("reading a qubit", span));
            }
        },
        Expr::Unary(UnOp::Neg, e) => Value::Int(Int::Sub(Box::new(Int::Lit(0)), int(e)?)),
        Expr::Unary(UnOp::Not, e) => Value::Bit(Cond::Not(bit(e)?)),
        Expr::Binary(a, op, b) => {
            let both_bits = matches!(roop_expr(l, a, span)?, Value::Bit(_));
            match (op, both_bits) {
                (BinOp::Add, _) => Value::Int(Int::Add(int(a)?, int(b)?)),
                (BinOp::Sub, _) => Value::Int(Int::Sub(int(a)?, int(b)?)),
                (BinOp::Mul, _) => Value::Int(Int::Mul(int(a)?, int(b)?)),
                (BinOp::Div, _) => Value::Int(Int::Div(int(a)?, int(b)?)),
                (BinOp::Rem, _) => Value::Int(Int::Rem(int(a)?, int(b)?)),
                (BinOp::BitAnd, false) => Value::Int(Int::And(int(a)?, int(b)?)),
                (BinOp::BitOr, false) => Value::Int(Int::Or(int(a)?, int(b)?)),
                (BinOp::And | BinOp::BitAnd, true) => Value::Bit(Cond::And(bit(a)?, bit(b)?)),
                (BinOp::Or | BinOp::BitOr, true) => Value::Bit(Cond::Or(bit(a)?, bit(b)?)),
                (BinOp::Eq, true) => Value::Bit(Cond::Not(Box::new(Cond::Xor(bit(a)?, bit(b)?)))),
                (BinOp::Ne, true) => Value::Bit(Cond::Xor(bit(a)?, bit(b)?)),
                (BinOp::Eq, false) => Value::Bit(Cond::Eq(*int(a)?, *int(b)?)),
                (BinOp::Ne, false) => Value::Bit(Cond::Not(Box::new(Cond::Eq(*int(a)?, *int(b)?)))),
                (BinOp::Le, false) => Value::Bit(Cond::Le(*int(a)?, *int(b)?)),
                (BinOp::Ge, false) => Value::Bit(Cond::Le(*int(b)?, *int(a)?)),
                (BinOp::Lt, false) => Value::Bit(Cond::Not(Box::new(Cond::Le(*int(b)?, *int(a)?)))),
                (BinOp::Gt, false) => Value::Bit(Cond::Not(Box::new(Cond::Le(*int(a)?, *int(b)?)))),
                _ => return Err(unsupported(&format!("`{op:?}` on these operands"), span)),
            }
        }
        _ => return Err(unsupported("this expression", span)),
    })
}

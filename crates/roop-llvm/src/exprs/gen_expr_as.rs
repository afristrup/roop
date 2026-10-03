use crate::{CodegenError, FnGen, Value, gen_expr};
use roop_syntax::{Expr, Type};

/// Like `gen_expr`, except that an integer literal takes the type it is
/// used with when that is `u8`.
pub fn gen_expr_as(g: &mut FnGen, expr: &Expr, expected: &Type) -> Result<Value, CodegenError> {
    if let (Expr::Int(n), Type::Named(name)) = (expr, expected)
        && name == "u8"
    {
        if !(0..=255).contains(n) {
            return Err(CodegenError::InvalidOperand("a u8 literal is 0 to 255"));
        }
        return Ok(Value {
            reg: n.to_string(),
            ty: expected.clone(),
        });
    }
    gen_expr(g, expr)
}

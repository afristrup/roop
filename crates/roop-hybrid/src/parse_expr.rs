use crate::{Cursor, Expr, HybridError, binary, parse_atom};

/// Precedence climbing over the operators of `binary`, from `min` up.
pub fn parse_expr(c: &mut Cursor, min: u8) -> Result<Expr, HybridError> {
    let mut lhs = parse_atom(c)?;
    while let Some((prec, right, build)) = binary(c.peek()) {
        if prec < min {
            break;
        }
        c.bump();
        let rhs = parse_expr(c, if right { prec } else { prec + 1 })?;
        lhs = build(Box::new(lhs), Box::new(rhs));
    }
    Ok(lhs)
}

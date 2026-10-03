use crate::{
    CodegenError, FnGen, Value, bool_type, gen_binary, gen_cast, gen_expr_as, gen_place, gen_unary,
    mem_load,
};
use roop_syntax::{Expr, Type};

pub fn gen_expr(g: &mut FnGen, expr: &Expr) -> Result<Value, CodegenError> {
    match expr {
        Expr::Int(i) => Ok(Value {
            reg: i.to_string(),
            ty: Type::Named("i64".into()),
        }),
        Expr::Float(f) => Ok(Value {
            reg: format!("0x{:016X}", f.to_bits()),
            ty: Type::Named("f64".into()),
        }),
        Expr::Byte(b) => Ok(Value {
            reg: b.to_string(),
            ty: Type::Named("u8".into()),
        }),
        Expr::Str(_) => Err(CodegenError::InvalidOperand(
            "a string literal is only an argument for a read-only array parameter",
        )),
        Expr::Cast(inner, ty) => {
            let v = gen_expr(g, inner)?;
            gen_cast(g, v, ty)
        }
        Expr::Empty => Err(CodegenError::InvalidOperand(
            "`empty` only starts an ancilla stack",
        )),
        Expr::Bool(b) => Ok(Value {
            reg: b.to_string(),
            ty: bool_type(),
        }),
        Expr::Variant(enum_name, variant) => {
            let index = g
                .ctx
                .enums
                .get(enum_name.as_str())
                .and_then(|def| def.variants.iter().position(|v| v == variant))
                .ok_or_else(|| CodegenError::UnknownName {
                    kind: "variant",
                    name: format!("{enum_name}::{variant}"),
                })?;
            Ok(Value {
                reg: index.to_string(),
                ty: Type::Named(enum_name.clone()),
            })
        }
        Expr::Place(place) => {
            let slot = gen_place(g, place)?;
            mem_load(g, &slot)
        }
        Expr::Unary(op, inner) => {
            let v = gen_expr(g, inner)?;
            gen_unary(g, *op, v)
        }
        Expr::Binary(lhs, op, rhs) => {
            let (l, r) = match (&**lhs, &**rhs) {
                (Expr::Int(_), Expr::Int(_)) => (gen_expr(g, lhs)?, gen_expr(g, rhs)?),
                (Expr::Int(_), _) => {
                    let r = gen_expr(g, rhs)?;
                    (gen_expr_as(g, lhs, &r.ty)?, r)
                }
                (_, Expr::Int(_)) => {
                    let l = gen_expr(g, lhs)?;
                    let r = gen_expr_as(g, rhs, &l.ty)?;
                    (l, r)
                }
                _ => (gen_expr(g, lhs)?, gen_expr(g, rhs)?),
            };
            gen_binary(g, l, *op, r)
        }
    }
}

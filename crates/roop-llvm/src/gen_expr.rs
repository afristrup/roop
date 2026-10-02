use super::{CodegenError, FnGen, Value, bool_type, gen_binary, gen_place, gen_unary, llvm_type};
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
            let ty = llvm_type(g.ctx, &slot.ty)?;
            let reg = format!("%{}", g.fresh("t"));
            g.emit(&format!("{reg} = load {ty}, ptr {}", slot.addr));
            Ok(Value { reg, ty: slot.ty })
        }
        Expr::Unary(op, inner) => {
            let v = gen_expr(g, inner)?;
            gen_unary(g, *op, v)
        }
        Expr::Binary(lhs, op, rhs) => {
            let l = gen_expr(g, lhs)?;
            let r = gen_expr(g, rhs)?;
            gen_binary(g, l, *op, r)
        }
    }
}

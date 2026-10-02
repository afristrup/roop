use super::{CodegenError, FnGen, Slot, Value, gen_assert, gen_expr, llvm_type, same_type};
use roop_syntax::{Place, Type};

pub fn gen_place(g: &mut FnGen, place: &Place) -> Result<Slot, CodegenError> {
    match place {
        Place::Var(name) => g
            .vars
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, slot)| slot.clone())
            .ok_or_else(|| CodegenError::UnknownName {
                kind: "variable",
                name: name.clone(),
            }),
        Place::Field(base, field) => {
            let base = gen_place(g, base)?;
            let Type::Named(owner) = &base.ty else {
                return Err(CodegenError::InvalidOperand("field access on a non-struct"));
            };
            let def =
                g.ctx
                    .structs
                    .get(owner.as_str())
                    .ok_or_else(|| CodegenError::UnknownName {
                        kind: "struct",
                        name: owner.clone(),
                    })?;
            let index = def
                .fields
                .iter()
                .position(|f| &f.name == field)
                .ok_or_else(|| CodegenError::UnknownName {
                    kind: "field",
                    name: field.clone(),
                })?;
            let addr = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{addr} = getelementptr inbounds %{owner}, ptr {}, i32 0, i32 {index}",
                base.addr
            ));
            Ok(Slot {
                addr,
                ty: def.fields[index].ty.clone(),
            })
        }
        Place::Index(base, index) => {
            let base = gen_place(g, base)?;
            let Type::Array(elem, len) = &base.ty else {
                return Err(CodegenError::InvalidOperand("indexing a non-array"));
            };
            let i = gen_expr(g, index)?;
            same_type(&Type::Named("i64".into()), &i.ty)?;
            let in_bounds = format!("%{}", g.fresh("t"));
            g.emit(&format!("{in_bounds} = icmp ult i64 {}, {len}", i.reg));
            gen_assert(
                g,
                &Value {
                    reg: in_bounds,
                    ty: Type::Named("bool".into()),
                },
            )?;
            let array_ty = llvm_type(g.ctx, &base.ty)?;
            let addr = format!("%{}", g.fresh("t"));
            g.emit(&format!(
                "{addr} = getelementptr inbounds {array_ty}, ptr {}, i64 0, i64 {}",
                base.addr, i.reg
            ));
            Ok(Slot {
                addr,
                ty: (**elem).clone(),
            })
        }
    }
}

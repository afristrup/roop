use crate::{CodegenError, Dialect, FnGen, Value, kind_of};
use roop_syntax::Type;

/// `e as T` between `i64`, `u8`, `f64` and `bool`. A float converts with
/// saturation, so a value out of range or not a number cannot be undefined.
pub fn gen_cast(g: &mut FnGen, v: Value, to: &Type) -> Result<Value, CodegenError> {
    if v.ty == *to {
        return Ok(v);
    }
    let name = |t: &Type| match t {
        Type::Named(n) => n.clone(),
        _ => String::new(),
    };
    kind_of(g.ctx, &v.ty)?;
    kind_of(g.ctx, to)?;
    let (from_name, to_name) = (name(&v.ty), name(to));
    let reg = format!("%{}", g.fresh("t"));
    let line = match (from_name.as_str(), to_name.as_str()) {
        ("i64", "u8") => format!("{reg} = trunc i64 {} to i8", v.reg),
        ("u8", "i64") => format!("{reg} = zext i8 {} to i64", v.reg),
        ("bool", "i64") => format!("{reg} = zext i1 {} to i64", v.reg),
        ("bool", "u8") => format!("{reg} = zext i1 {} to i8", v.reg),
        ("i64", "f64") => format!("{reg} = sitofp i64 {} to double", v.reg),
        ("u8", "f64") => format!("{reg} = uitofp i8 {} to double", v.reg),
        ("f64", "i64") => format!(
            "{reg} = call i64 @llvm.fptosi.sat.i64.f64(double {})",
            v.reg
        ),
        ("f64", "u8") => format!("{reg} = call i8 @llvm.fptoui.sat.i8.f64(double {})", v.reg),
        _ => return Err(CodegenError::InvalidOperand("no such conversion")),
    };
    if g.dialect != Dialect::Host && line.contains("sat") {
        return Err(CodegenError::Unsupported("a float conversion in a kernel"));
    }
    g.emit(&line);
    Ok(Value {
        reg,
        ty: to.clone(),
    })
}

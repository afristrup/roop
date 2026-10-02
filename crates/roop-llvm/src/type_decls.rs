use super::{CodegenError, Ctx, llvm_type};
use roop_syntax::{Item, Program};

pub fn type_decls(ctx: &Ctx, program: &Program) -> Result<String, CodegenError> {
    let mut out = String::new();
    for item in &program.items {
        let Item::Struct(def) = item else { continue };
        let fields = def
            .fields
            .iter()
            .map(|f| llvm_type(ctx, &f.ty))
            .collect::<Result<Vec<_>, _>>()?;
        out.push_str(&format!(
            "%{} = type {{ {} }}\n",
            def.name,
            fields.join(", ")
        ));
    }
    Ok(out)
}

use crate::Ctx;

/// Attribute group reference appended to every `define`.
pub fn function_attrs(ctx: &Ctx) -> &'static str {
    if ctx.options.cpu.is_some() { " #0" } else { "" }
}

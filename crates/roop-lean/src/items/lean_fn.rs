use crate::{
    Ctx, Dir, Env, LeanError, Out, esc, is_mut_ref, lean_block, lean_type, tuple_expr, tuple_type,
};
use roop_syntax::FnDef;

/// The function as a pure Lean definition: its mutable parameters go in and
/// come back as the result. Backward it is the inverse function `f_inv`.
/// Returns the text and how many loops it contains.
pub fn lean_fn(cx: &Ctx, def: &FnDef, dir: Dir) -> Result<(String, usize), LeanError> {
    let name = match dir {
        Dir::Forward => def.name.clone(),
        Dir::Backward => format!("{}_inv", def.name),
    };
    let mut env = Env {
        vars: def
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect(),
        irreversible: def.irreversible,
    };
    let mutable: Vec<_> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    let result_types: Vec<_> = mutable.iter().map(|p| p.ty.clone()).collect();
    let params: Vec<String> = def
        .params
        .iter()
        .map(|p| format!("({} : {})", esc(&p.name), lean_type(&p.ty)))
        .collect();

    let mut out = Out::default();
    out.indent = 1;
    for p in &def.params {
        out.line(&format!("let mut {} := {}", esc(&p.name), esc(&p.name)));
    }
    lean_block(cx, &mut env, &mut out, &def.body, dir)?;
    let returned: Vec<String> = mutable.iter().map(|p| esc(&p.name)).collect();
    out.line(&format!("return {}", tuple_expr(&returned)));
    let text = format!(
        "def {} {} : Roop.Res {} := do\n{}\n",
        esc(&name),
        params.join(" "),
        tuple_type(&result_types),
        out.text
    );
    Ok((text, out.loops))
}

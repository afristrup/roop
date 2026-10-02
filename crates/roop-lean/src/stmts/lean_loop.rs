use crate::{
    Ctx, Dir, Env, LeanError, Out, assign_place, esc, lean_block, lean_expr, tuple_expr,
    tuple_proj, tuple_type,
};
use roop_check::body_effects;
use roop_syntax::{Block, Expr, Place, Type};

/// A reversible loop as the prelude's `Roop.janus` over the tuple of variables
/// the loop writes. Backward swaps entry and exit and runs both blocks
/// backward, which is the loop's inverse.
pub fn lean_loop(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    out.loops += 1;
    let (mut written, step_written) = (body_effects(body).writes, body_effects(step).writes);
    written.extend(step_written);
    let mut state: Vec<(String, Type)> = Vec::new();
    for (name, ty) in &env.vars {
        if written.contains(name) && !state.iter().any(|(n, _)| n == name) {
            state.push((name.clone(), ty.clone()));
        }
    }
    let types: Vec<Type> = state.iter().map(|(_, t)| unref(t)).collect();
    let state_type = tuple_type(&types);
    let (start, first, second, stop) = match dir {
        Dir::Forward => (entry, body, step, until),
        Dir::Backward => (until, body, step, entry),
    };

    let [entry_fn, stop_fn, body_fn, step_fn] = [
        out.fresh("__entry"),
        out.fresh("__stop"),
        out.fresh("__body"),
        out.fresh("__step"),
    ];
    for (name, condition) in [(&entry_fn, start), (&stop_fn, stop)] {
        out.line(&format!(
            "let {name} : {state_type} \u{2192} Roop.Res Bool := fun s => do"
        ));
        out.indent += 1;
        unpack(out, &state);
        out.line(&format!("return {}", lean_expr(cx, env, condition)?));
        out.indent -= 1;
    }
    for (name, block) in [(&body_fn, first), (&step_fn, second)] {
        out.line(&format!(
            "let {name} : {state_type} \u{2192} Roop.Res {state_type} := fun s => do"
        ));
        out.indent += 1;
        unpack(out, &state);
        lean_block(cx, env, out, block, dir)?;
        let names: Vec<String> = state.iter().map(|(n, _)| esc(n)).collect();
        out.line(&format!("return {}", tuple_expr(&names)));
        out.indent -= 1;
    }
    let names: Vec<String> = state.iter().map(|(n, _)| esc(n)).collect();
    let result = out.fresh("__loop");
    out.line(&format!(
        "let {result} \u{2190} Roop.janus {entry_fn} {stop_fn} {body_fn} {step_fn} {}",
        tuple_expr(&names)
    ));
    for (k, (name, _)) in state.iter().enumerate() {
        let place = Place::Var(name.clone());
        assign_place(cx, env, out, &place, &tuple_proj(&result, k, state.len()))?;
    }
    Ok(())
}

fn unref(ty: &Type) -> Type {
    match ty {
        Type::Ref { inner, .. } => (**inner).clone(),
        other => other.clone(),
    }
}

/// Opens the state tuple `s` into mutable variables inside a closure.
fn unpack(out: &mut Out, state: &[(String, Type)]) {
    let names: Vec<String> = state.iter().map(|(n, _)| esc(n)).collect();
    match names.as_slice() {
        [] => {}
        [only] => out.line(&format!("let mut {only} := s")),
        many => {
            out.line(&format!("let ({}) := s", many.join(", ")));
            for name in many {
                out.line(&format!("let mut {name} := {name}"));
            }
        }
    }
}

use crate::{
    Construct, Ctx, Dir, Env, LeanError, Lifted, Out, Piece, assign_place, esc, esc_fn, is_bool,
    lean_piece, lean_zero, place_type, read_place, tuple_expr, tuple_proj,
};
use roop_check::body_effects;
use roop_syntax::{Block, Place};

/// `try { body } catch_rollback { handler } -> outcome;` as the prelude's
/// `Roop.tryCatch`, an exception monad whose outcome is kept as data. The model
/// is pure, so a failed body leaves no trace and the handler runs on the
/// original state. Backward, `Roop.untry` picks the side to undo by the outcome.
#[allow(clippy::too_many_arguments)] // mirrors the fields of the statement
pub fn lean_try(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    at: usize,
    body: &Block,
    handler: &Block,
    outcome: &Place,
    dir: Dir,
) -> Result<(), LeanError> {
    let mut written = body_effects(body).writes;
    written.extend(body_effects(handler).writes);
    written.extend(env.logged.iter().map(|h| place_root(h).to_string()));
    let (state, captures): (Vec<_>, Vec<_>) = env
        .visible()
        .into_iter()
        .partition(|(name, _)| written.contains(name));
    let id = format!("{}__try{at}", env.function);
    let info = Lifted {
        id: id.clone(),
        construct: Construct::Try,
        captures,
        state,
        parallel: None,
        one_way: true,
    };
    let piece = |part: &str| esc_fn(&format!("{id}_{part}"));
    let args: Vec<String> = info.captures.iter().map(|(n, _)| esc(n)).collect();
    let applied = |part: &str| format!("({} {})", piece(part), args.join(" "));
    let names: Vec<String> = info.state.iter().map(|(n, _)| esc(n)).collect();
    let count = info.state.len();

    let ty = place_type(cx, env, outcome)?;
    let zero = lean_zero(&ty)?;
    let read = read_place(cx, env, outcome)?;
    let raised = if is_bool(&ty) {
        read.clone()
    } else {
        format!("({read} != {zero})")
    };
    let result = out.fresh("__try");
    match dir {
        Dir::Forward => {
            lean_piece(cx, env, out, &info, &piece("body"), Piece::Block(body), dir)?;
            lean_piece(
                cx,
                env,
                out,
                &info,
                &piece("handler"),
                Piece::Block(handler),
                dir,
            )?;
            out.line(&format!(
                "Roop.check ({read} == {zero}) Roop.Fail.assertion"
            ));
            out.line(&format!(
                "let {result} \u{2190} Roop.tryCatch {} {} {}",
                applied("body"),
                applied("handler"),
                tuple_expr(&names)
            ));
            let state_part = format!("{result}.1");
            for (k, (name, _)) in info.state.iter().enumerate() {
                let place = Place::Var(name.clone());
                assign_place(cx, env, out, &place, &tuple_proj(&state_part, k, count))?;
            }
            let tag = format!("{result}.2");
            let value = if is_bool(&ty) {
                tag
            } else {
                format!("(if {tag} then (1 : Roop.I64) else 0)")
            };
            assign_place(cx, env, out, outcome, &value)?;
        }
        Dir::Backward => {
            lean_piece(
                cx,
                env,
                out,
                &info,
                &piece("body_inv"),
                Piece::Block(body),
                dir,
            )?;
            lean_piece(
                cx,
                env,
                out,
                &info,
                &piece("handler_inv"),
                Piece::Block(handler),
                dir,
            )?;
            out.line(&format!(
                "let {result} \u{2190} Roop.untry {} {} {} {raised}",
                applied("body_inv"),
                applied("handler_inv"),
                tuple_expr(&names)
            ));
            for (k, (name, _)) in info.state.iter().enumerate() {
                let place = Place::Var(name.clone());
                assign_place(cx, env, out, &place, &tuple_proj(&result, k, count))?;
            }
            assign_place(cx, env, out, outcome, &zero)?;
        }
    }
    out.pieces.push(info);
    Ok(())
}

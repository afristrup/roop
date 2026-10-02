use crate::{
    Construct, Ctx, Dir, Env, LeanError, Lifted, Out, Piece, assign_place, esc, esc_fn, lean_piece,
    tuple_expr, tuple_proj,
};
use roop_check::body_effects;
use roop_syntax::{Block, Expr, Place, counted_loop};

/// A reversible loop as the prelude's `Roop.janus` over the tuple of variables
/// the loop writes. Its pieces are lifted into top-level definitions: forward
/// defines all four, backward only the inverse body and step. Backward swaps
/// entry and exit and runs both blocks backward, which is the loop's inverse.
#[allow(clippy::too_many_arguments)] // mirrors the fields of the statement
pub fn lean_loop(
    cx: &Ctx,
    env: &mut Env,
    out: &mut Out,
    at: usize,
    parallel: bool,
    entry: &Expr,
    body: &Block,
    step: &Block,
    until: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    let mut written = body_effects(body).writes;
    written.extend(body_effects(step).writes);
    let (state, captures): (Vec<_>, Vec<_>) = env
        .visible()
        .into_iter()
        .partition(|(name, _)| written.contains(name));
    let id = format!("{}__loop{at}", env.function);
    let parallel = parallel
        .then(|| counted_loop(entry, step, until))
        .flatten()
        .map(|counted| counted.var.to_string());
    let before = out.pieces.len();
    let mut info = Lifted {
        id: id.clone(),
        construct: Construct::Loop,
        captures,
        state,
        parallel,
        one_way: false,
    };
    let piece = |part: &str| esc_fn(&format!("{id}_{part}"));
    let args: Vec<String> = info.captures.iter().map(|(n, _)| esc(n)).collect();
    let applied = |part: &str| format!("({} {})", piece(part), args.join(" "));

    let janus = match dir {
        Dir::Forward => {
            lean_piece(
                cx,
                env,
                out,
                &info,
                &piece("entry"),
                Piece::Condition(entry),
                dir,
            )?;
            lean_piece(
                cx,
                env,
                out,
                &info,
                &piece("stop"),
                Piece::Condition(until),
                dir,
            )?;
            lean_piece(cx, env, out, &info, &piece("body"), Piece::Block(body), dir)?;
            lean_piece(cx, env, out, &info, &piece("step"), Piece::Block(step), dir)?;
            format!(
                "{} {} {} {}",
                applied("entry"),
                applied("stop"),
                applied("body"),
                applied("step")
            )
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
                &piece("step_inv"),
                Piece::Block(step),
                dir,
            )?;
            format!(
                "{} {} {} {}",
                applied("stop"),
                applied("entry"),
                applied("body_inv"),
                applied("step_inv")
            )
        }
    };

    let names: Vec<String> = info.state.iter().map(|(n, _)| esc(n)).collect();
    let result = out.fresh("__loop");
    out.line(&format!(
        "let {result} \u{2190} Roop.janus {janus} {}",
        tuple_expr(&names)
    ));
    for (k, (name, _)) in info.state.iter().enumerate() {
        let place = Place::Var(name.clone());
        assign_place(
            cx,
            env,
            out,
            &place,
            &tuple_proj(&result, k, info.state.len()),
        )?;
    }
    info.one_way = out.pieces[before..]
        .iter()
        .any(|p| p.construct == Construct::Try || p.one_way);
    out.pieces.push(info);
    Ok(())
}

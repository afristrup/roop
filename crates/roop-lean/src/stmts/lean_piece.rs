use crate::{
    Ctx, Dir, Env, LeanError, LoopInfo, Out, esc, lean_block, lean_expr, lean_type, tuple_expr,
    tuple_type, unpack_state, unref,
};
use roop_syntax::{Block, Expr};

pub enum Piece<'a> {
    Condition(&'a Expr),
    Block(&'a Block),
}

/// Lifts one piece of a loop into a top-level definition over the captured
/// variables and the state tuple, so theorems can talk about it by name.
pub fn lean_piece(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    info: &LoopInfo,
    name: &str,
    piece: Piece,
    dir: Dir,
) -> Result<(), LeanError> {
    let mut inner_env = Env {
        vars: info
            .captures
            .iter()
            .chain(&info.state)
            .cloned()
            .collect(),
        irreversible: env.irreversible,
        function: env.function.clone(),
    };
    let mut inner = Out::default();
    inner.indent = 1;
    unpack_state(&mut inner, &info.state);
    let state_type = tuple_type(
        &info
            .state
            .iter()
            .map(|(_, t)| unref(t))
            .collect::<Vec<_>>(),
    );
    let result = match piece {
        Piece::Condition(cond) => {
            inner.line(&format!("return {}", lean_expr(cx, &inner_env, cond)?));
            "Bool".to_string()
        }
        Piece::Block(block) => {
            lean_block(cx, &mut inner_env, &mut inner, block, dir)?;
            let names: Vec<String> = info.state.iter().map(|(n, _)| esc(n)).collect();
            inner.line(&format!("return {}", tuple_expr(&names)));
            state_type.clone()
        }
    };
    let captures: Vec<String> = info
        .captures
        .iter()
        .map(|(n, t)| format!("({} : {})", esc(n), lean_type(t)))
        .collect();
    out.lifted.push_str(&inner.lifted);
    out.loops.extend(inner.loops);
    out.ancillas += inner.ancillas;
    out.lifted.push_str(&format!(
        "def {name} {} (s : {state_type}) : Roop.Res {result} := do\n{}\n",
        captures.join(" "),
        inner.text
    ));
    Ok(())
}

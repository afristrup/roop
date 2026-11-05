use crate::{
    AdditiveUpdate, Ctx, Dir, Env, LeanError, Lifted, Out, ScalarAdditiveUpdate, esc, esc_fn,
    esc_thm, lean_block, lean_expr, lean_type, place_type, tuple_expr, tuple_type, unpack_state,
    unref,
};
use roop_syntax::{Block, Expr, Place, StmtKind, Type, UpdateOp};

pub enum Piece<'a> {
    Condition(&'a Expr),
    Block(&'a Block),
}

pub fn lean_piece(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    info: &mut Lifted,
    name: &str,
    piece: Piece,
    dir: Dir,
) -> Result<(), LeanError> {
    if let Piece::Block(block) = piece
        && let Some((target, index, value)) = additive_update(cx, env, info, block)?
    {
        return lean_additive_piece(cx, env, out, info, name, target, index, value, dir);
    }
    if let Piece::Block(block) = piece
        && scalar_additive_update(cx, env, info, block)?
    {
        info.scalar_additive_update = Some(ScalarAdditiveUpdate);
    }
    let mut inner_env = Env {
        vars: info.captures.iter().chain(&info.state).cloned().collect(),
        irreversible: env.irreversible,
        function: env.function.clone(),
        logged: env.logged.clone(),
    };
    let mut inner = Out::default();
    inner.indent = 1;
    unpack_state(&mut inner, &info.state);
    let state_type = tuple_type(
        &info
            .state
            .iter()
            .map(|(_, ty)| unref(ty))
            .collect::<Vec<_>>(),
    );
    let result = match piece {
        Piece::Condition(cond) => {
            inner.line(&format!("return {}", lean_expr(cx, &inner_env, cond)?));
            "Bool".to_string()
        }
        Piece::Block(block) => {
            lean_block(cx, &mut inner_env, &mut inner, block, dir)?;
            let names: Vec<String> = info.state.iter().map(|(name, _)| esc(name)).collect();
            inner.line(&format!("return {}", tuple_expr(&names)));
            state_type.clone()
        }
    };
    let captures: Vec<String> = info
        .captures
        .iter()
        .map(|(name, ty)| format!("({} : {})", esc(name), lean_type(ty)))
        .collect();
    out.lifted.push_str(&inner.lifted);
    out.pieces.extend(inner.pieces);
    out.ancillas += inner.ancillas;
    out.lifted.push_str(&format!(
        "def {name} {} (__st : {state_type}) : Roop.Res {result} := do\n{}\n",
        captures.join(" "),
        inner.text
    ));
    Ok(())
}

fn additive_update<'a>(
    cx: &Ctx,
    env: &Env,
    info: &Lifted,
    block: &'a Block,
) -> Result<Option<(&'a str, &'a str, &'a Expr)>, LeanError> {
    if block.stmts.len() != 1 {
        return Ok(None);
    }
    let stmt = &block.stmts[0];
    let StmtKind::Update {
        target: target_place @ Place::Index(base, index),
        op: UpdateOp::Add,
        value,
    } = &stmt.kind
    else {
        return Ok(None);
    };
    let Place::Var(root) = &**base else {
        return Ok(None);
    };
    let Some((state, _)) = info.state.first() else {
        return Ok(None);
    };
    if root != state
        || info.state.len() != 2
        || place_type(cx, env, target_place)? != Type::Named("i64".into())
    {
        return Ok(None);
    }
    let Expr::Place(Place::Var(index_name)) = &**index else {
        return Ok(None);
    };
    let index_is_state =
        info.state[1].0 == *index_name && info.state[1].1 == Type::Named("i64".into());
    if (!info.captures.iter().any(|(name, _)| name == index_name) && !index_is_state)
        || expr_uses(value, root)
    {
        return Ok(None);
    }
    Ok(Some((root, index_name, value)))
}

fn scalar_additive_update(
    cx: &Ctx,
    env: &Env,
    info: &Lifted,
    block: &Block,
) -> Result<bool, LeanError> {
    if block.stmts.len() != 1 || info.state.len() != 2 {
        return Ok(false);
    }
    let stmt = &block.stmts[0];
    let StmtKind::Update {
        target: target_place @ Place::Var(target),
        op: UpdateOp::Add,
        value,
    } = &stmt.kind
    else {
        return Ok(false);
    };
    Ok(info.state[1].0 == *target
        && place_type(cx, env, target_place)? == Type::Named("i64".into())
        && !expr_uses(value, target))
}

fn args_with(captures: &[(String, Type)]) -> String {
    captures
        .iter()
        .map(|(name, _)| esc(name))
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_count(expr: &Expr) -> usize {
    match expr {
        Expr::Place(place) => place_reads(place),
        Expr::Unary(_, expr) | Expr::Cast(expr, _) => read_count(expr),
        Expr::Binary(left, _, right) => read_count(left) + read_count(right),
        _ => 0,
    }
}

fn place_reads(place: &Place) -> usize {
    match place {
        Place::Var(_) => 0,
        Place::Field(base, _) => place_reads(base),
        Place::Index(base, index) => place_reads(base) + read_count(index) + 1,
    }
}

fn expr_uses(expr: &Expr, root: &str) -> bool {
    match expr {
        Expr::Place(place) => place_uses(place, root),
        Expr::Unary(_, expr) | Expr::Cast(expr, _) => expr_uses(expr, root),
        Expr::Binary(left, _, right) => expr_uses(left, root) || expr_uses(right, root),
        _ => false,
    }
}

fn place_uses(place: &Place, root: &str) -> bool {
    match place {
        Place::Var(name) => name == root,
        Place::Field(base, _) => place_uses(base, root),
        Place::Index(base, index) => place_uses(base, root) || expr_uses(index, root),
    }
}

#[allow(clippy::too_many_arguments)]
fn lean_additive_piece(
    cx: &Ctx,
    env: &Env,
    out: &mut Out,
    info: &mut Lifted,
    name: &str,
    target: &str,
    index: &str,
    value: &Expr,
    dir: Dir,
) -> Result<(), LeanError> {
    let (_, preserved_ty) = &info.state[1];
    let delta = esc_fn(&format!("{}_body_delta", info.id));
    let delta_no_ancilla = esc_thm(&format!("{}_body_delta_no_ancilla", info.id));
    if dir == Dir::Forward {
        let delta_env = Env {
            vars: info
                .captures
                .iter()
                .chain(info.state.iter().skip(1))
                .cloned()
                .collect(),
            irreversible: env.irreversible,
            function: env.function.clone(),
            logged: env.logged.clone(),
        };
        let captures: Vec<String> = info
            .captures
            .iter()
            .map(|(name, ty)| format!("({} : {})", esc(name), lean_type(ty)))
            .collect();
        out.lifted.push_str(&format!(
            "def {delta} {} ({} : {}) : Roop.Res Roop.I64 := do\n  return {}\n",
            captures.join(" "),
            esc(&info.state[1].0),
            lean_type(preserved_ty),
            lean_expr(cx, &delta_env, value)?,
        ));
        out.lifted.push_str(&format!(
            "theorem {delta_no_ancilla} {} ({} : {}) : {delta} {} {} ≠ Except.error Roop.Fail.ancilla := by\n  simp only [{delta}]\n{}  all_goals simp [pure, Except.pure]\n",
            captures.join(" "),
            esc(&info.state[1].0),
            lean_type(preserved_ty),
            args_with(&info.captures),
            esc(&info.state[1].0),
            "  all_goals first | (apply Roop.bind_ne_ancilla (by apply Roop.aget_ne_ancilla); intro _) | apply Roop.aget_ne_ancilla\n".repeat(read_count(value)),
        ));
        info.additive_update = Some(AdditiveUpdate {
            delta: delta.clone(),
            index: esc(index),
            index_is_state: info.state[1].0 == index,
        });
    }
    let mut inner = Out::default();
    inner.indent = 1;
    unpack_state(&mut inner, &info.state);
    let state_type = tuple_type(
        &info
            .state
            .iter()
            .map(|(_, ty)| unref(ty))
            .collect::<Vec<_>>(),
    );
    let args: Vec<String> = info.captures.iter().map(|(name, _)| esc(name)).collect();
    let applied = format!("{delta} {} {}", args.join(" "), esc(&info.state[1].0));
    let old = format!("Roop.aget {} {}", esc(target), esc(index));
    let op = if dir == Dir::Forward { "+" } else { "-" };
    inner.line(&format!("let __old \u{2190} {old}"));
    inner.line(&format!("let __delta \u{2190} {applied}"));
    inner.line(&format!(
        "{}\u{2190} Roop.aset {} {} (__old {op} __delta)",
        esc(target),
        esc(target),
        esc(index),
    ));
    let names: Vec<String> = info.state.iter().map(|(name, _)| esc(name)).collect();
    inner.line(&format!("return {}", tuple_expr(&names)));
    let captures: Vec<String> = info
        .captures
        .iter()
        .map(|(name, ty)| format!("({} : {})", esc(name), lean_type(ty)))
        .collect();
    out.lifted.push_str(&format!(
        "def {name} {} (__st : {state_type}) : Roop.Res {state_type} := do\n{}\n",
        captures.join(" "),
        inner.text
    ));
    Ok(())
}

use crate::{
    LoopInfo, LoopLemmas, esc, esc_fn, esc_thm, lean_type, proof_script, tuple_expr, tuple_proj,
    unfold_simp, unref,
};

/// A `#[parallel]` loop runs its iterations in any order, so any two of them
/// must commute: running iteration `v` then `w` ends in the same state as `w`
/// then `v`, whatever the starting state. This is what the checker's
/// disjointness rules promise, stated as a theorem about the loop body. The
/// loop variable is reset at the end, since it is the only thing that differs.
pub fn lean_commute(info: &LoopInfo, deps: &[String], earlier: &LoopLemmas) -> Option<String> {
    let var = info.parallel.as_ref()?;
    let at = info.state.iter().position(|(name, _)| name == var)?;
    let n = info.state.len();
    let id = &info.id;
    let body = format!(
        "({} {})",
        esc_fn(&format!("{id}_body")),
        info.captures
            .iter()
            .map(|(name, _)| esc(name))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let explicit: Vec<String> = info
        .captures
        .iter()
        .map(|(name, ty)| format!("({} : {})", esc(name), lean_type(ty)))
        .collect();
    let components: Vec<String> = (1..=n).map(|k| format!("\u{ab}__s{k}\u{bb}")).collect();
    let typed: Vec<String> = components
        .iter()
        .zip(&info.state)
        .map(|(name, (_, ty))| format!("({name} : {})", lean_type(&unref(ty))))
        .collect();
    let with = |source: &dyn Fn(usize) -> String, value: &str| {
        let items: Vec<String> = (0..n)
            .map(|k| if k == at { value.into() } else { source(k) })
            .collect();
        tuple_expr(&items)
    };
    let start = |k: usize| components[k].clone();
    let after = |name: &'static str| move |k: usize| tuple_proj(name, k, n);
    let order = |first: &str, second: &str| {
        format!(
            "(do let t \u{2190} {body} {}; let u \u{2190} {body} {}; pure {})",
            with(&start, first),
            with(&after("t"), second),
            with(&after("u"), "(0 : Roop.I64)"),
        )
    };
    let mut unfold = vec![esc_fn(&format!("{id}_body"))];
    unfold.extend(deps.iter().cloned());
    let (v, w) = ("\u{ab}__v\u{bb}", "\u{ab}__w\u{bb}");
    Some(format!(
        "theorem {} {} {} ({v} {w} : Roop.I64) (hvw : {v} \u{2260} {w}) :\n    {} =\n    {} := by\n  have hne : {v}.toNat \u{2260} {w}.toNat := fun h => hvw (BitVec.eq_of_toNat_eq h)\n{}{}\n",
        esc_thm(&format!("{id}_commute")),
        explicit.join(" "),
        typed.join(" "),
        order(v, w),
        order(w, v),
        unfold_simp(&unfold, "\u{22a2}"),
        proof_script(earlier),
    ))
}

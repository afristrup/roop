use crate::{HYP, LemmaShape, Lifted, LoopLemmas, esc_fn, proof_script, unfold_simp};

/// A lifted piece never fails on an unrestored ancilla.
pub fn no_ancilla_lemma(
    info: &Lifted,
    shape: &LemmaShape,
    part: &str,
    deps: &[String],
    earlier: &LoopLemmas,
) -> String {
    let a = "\u{ab}__a\u{bb}";
    let piece = esc_fn(&format!("{}_{part}", shape.id));
    let mut unfold = vec![piece.clone()];
    if matches!(part, "body" | "body_inv")
        && let Some(update) = &info.additive_update
    {
        unfold.push(update.delta.clone());
    }
    unfold.extend(deps.iter().cloned());
    let simple = format!(
        "  simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at {HYP} <;> done\n",
        unfold.join(", ")
    );
    format!(
        "theorem {} {} ({a} : {}) ({HYP} : {piece} {} {a} = Except.error Roop.Fail.ancilla) : False := by\n  first\n  | ({simple})\n  | (\n{}{}{})\n",
        shape.theorem(&format!("{part}_no_ancilla")),
        shape.explicit,
        shape.state,
        shape.args,
        shape.destructure(a),
        unfold_simp(&unfold, HYP, false),
        proof_script(earlier, 0),
    )
}

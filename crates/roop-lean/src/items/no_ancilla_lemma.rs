use crate::{HYP, LemmaShape, Lifted, LoopLemmas, esc_fn, esc_thm, proof_script, unfold_simp};

/// A lifted piece never fails on an unrestored ancilla.
pub fn no_ancilla_lemma(
    info: &Lifted,
    shape: &LemmaShape,
    part: &str,
    deps: &[String],
    earlier: &LoopLemmas,
) -> String {
    if matches!(part, "body" | "body_inv")
        && let Some(update) = &info.additive_update
    {
        let a = "«__a»";
        let piece = esc_fn(&format!("{}_{part}", shape.id));
        let index = if update.index_is_state {
            "(fun z => z)".to_string()
        } else {
            format!("(fun _ => {})", update.index)
        };
        let delta = format!("({} {})", update.delta, shape.args);
        let delta_no_ancilla = esc_thm(&format!("{}_body_delta_no_ancilla", info.id));
        let theorem = if part == "body" {
            "aget_aset_add_no_ancilla"
        } else {
            "aget_aset_sub_no_ancilla"
        };
        let certificate = format!("({delta_no_ancilla} {} a2)", shape.args);
        return format!(
            "theorem {} {} ({a} : {}) ({HYP} : {piece} {} {a} = Except.error Roop.Fail.ancilla) : False := by\n  obtain ⟨a1, a2⟩ := {a}\n  have h := Roop.{theorem} a1 {index} {delta} a2 {certificate}\n  apply h\n  simpa only [{piece}] using {HYP}\n",
            shape.theorem(&format!("{part}_no_ancilla")),
            shape.explicit,
            shape.state,
            shape.args,
        );
    }
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

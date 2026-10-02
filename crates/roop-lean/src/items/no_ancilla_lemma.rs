use crate::{HYP, LemmaShape, LoopLemmas, esc_fn, proof_script, unfold_simp};

/// A lifted piece never fails on an unrestored ancilla.
pub fn no_ancilla_lemma(
    shape: &LemmaShape,
    part: &str,
    deps: &[String],
    earlier: &LoopLemmas,
) -> String {
    let a = "\u{ab}__a\u{bb}";
    let piece = esc_fn(&format!("{}_{part}", shape.id));
    let mut unfold = vec![piece.clone()];
    unfold.extend(deps.iter().cloned());
    format!(
        "theorem {} {} ({a} : {}) ({HYP} : {piece} {} {a} = Except.error Roop.Fail.ancilla) : False := by\n{}{}{}\n",
        shape.theorem(&format!("{part}_no_ancilla")),
        shape.explicit,
        shape.state,
        shape.args,
        shape.destructure(a),
        unfold_simp(&unfold, HYP),
        proof_script(earlier),
    )
}

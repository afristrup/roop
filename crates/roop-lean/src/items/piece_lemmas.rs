use crate::{LemmaShape, LoopLemmas, esc_fn, proof_script, unfold_simp};

/// A lifted piece and its inverse undo each other: `inv_f` says the inverse
/// restores what the piece produced, and with `both` so does the piece for
/// what the inverse produced.
pub fn piece_lemmas(
    shape: &LemmaShape,
    part: &str,
    both: bool,
    deps: &[String],
    earlier: &LoopLemmas,
) -> String {
    let (a, b) = ("\u{ab}__a\u{bb}", "\u{ab}__b\u{bb}");
    let id = &shape.id;
    let (fwd, inv) = (
        esc_fn(&format!("{id}_{part}")),
        esc_fn(&format!("{id}_{part}_inv")),
    );
    let mut unfold = vec![fwd.clone(), inv.clone()];
    unfold.extend(deps.iter().cloned());
    let directions: &[(&str, &String, &String)] = if both {
        &[("inv_f", &fwd, &inv), ("f_inv", &inv, &fwd)]
    } else {
        &[("inv_f", &fwd, &inv)]
    };
    let mut text = String::new();
    for (suffix, from, to) in directions {
        text.push_str(&format!(
            "theorem {} {} ({a} {b} : {}) (h : {from} {} {a} = Except.ok {b}) :\n    {to} {} {b} = Except.ok {a} := by\n{}{}{}\n",
            shape.theorem(&format!("{part}_{suffix}")),
            shape.explicit,
            shape.state,
            shape.args,
            shape.args,
            shape.destructure(a),
            unfold_simp(&unfold, "h \u{22a2}"),
            proof_script(earlier),
        ));
    }
    text
}

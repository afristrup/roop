use crate::{LoopInfo, esc, esc_fn, esc_thm, lean_type, proof_script, tuple_type, unfold_simp, unref};

/// Lemmas about one loop whose pieces were lifted out of its function: each
/// body and step is undone by its inverse, and therefore the loop is, by the
/// prelude's `Roop.janus_inv`. Returns the text and the names of the loop
/// lemmas, which later proofs use whenever a loop's result is known.
pub fn lean_loop_lemmas(info: &LoopInfo, deps: &[String], earlier: &[String]) -> (String, Vec<String>) {
    let id = &info.id;
    let args: Vec<String> = info.captures.iter().map(|(n, _)| esc(n)).collect();
    let applied = |part: &str| format!("({} {})", esc_fn(&format!("{id}_{part}")), args.join(" "));
    let explicit: Vec<String> = info
        .captures
        .iter()
        .map(|(n, t)| format!("({} : {})", esc(n), lean_type(t)))
        .collect();
    let implicit: Vec<String> = info
        .captures
        .iter()
        .map(|(n, t)| format!("{{{} : {}}}", esc(n), lean_type(t)))
        .collect();
    let state = tuple_type(&info.state.iter().map(|(_, t)| unref(t)).collect::<Vec<_>>());
    let (a, b) = ("\u{ab}__a\u{bb}", "\u{ab}__b\u{bb}");
    let destructure = |var: &str| match info.state.len() {
        0 | 1 => String::new(),
        n => {
            let parts: Vec<String> = (1..=n)
                .map(|k| format!("\u{ab}{}{k}\u{bb}", var.trim_matches(['\u{ab}', '\u{bb}'])))
                .collect();
            format!("  obtain \u{27e8}{}\u{27e9} := {var}\n", parts.join(", "))
        }
    };

    let mut text = String::new();
    for part in ["body", "step"] {
        let (fwd, inv) = (esc_fn(&format!("{id}_{part}")), esc_fn(&format!("{id}_{part}_inv")));
        let mut unfold = vec![fwd.clone(), inv.clone()];
        unfold.extend(deps.iter().cloned());
        for (suffix, from, to) in [("inv_f", &fwd, &inv), ("f_inv", &inv, &fwd)] {
            text.push_str(&format!(
                "theorem {} {} ({a} {b} : {state}) (h : {from} {} {a} = Except.ok {b}) :\n    {to} {} {b} = Except.ok {a} := by\n{}{}{}{}\n",
                esc_thm(&format!("{id}_{part}_{suffix}")),
                explicit.join(" "),
                args.join(" "),
                args.join(" "),
                destructure(a),
                destructure(b),
                unfold_simp(&unfold, "h \u{22a2}"),
                proof_script(earlier),
            ));
        }
    }

    let [inv_f, f_inv] = ["inv_f", "f_inv"].map(|s| esc_thm(&format!("{id}_loop_{s}")));
    let (s, r) = ("\u{ab}__s\u{bb}", "\u{ab}__r\u{bb}");
    let forward = format!(
        "Roop.janus {} {} {} {}",
        applied("entry"),
        applied("stop"),
        applied("body"),
        applied("step")
    );
    let backward = format!(
        "Roop.janus {} {} {} {}",
        applied("stop"),
        applied("entry"),
        applied("body_inv"),
        applied("step_inv")
    );
    let lemma_of = |part: &str, suffix: &str| format!("({} {})", esc_thm(&format!("{id}_{part}_{suffix}")), args.join(" "));
    text.push_str(&format!(
        "theorem {inv_f} {} {{{s} {r} : {state}}} (h : {forward} {s} = Except.ok {r}) :\n    {backward} {r} = Except.ok {s} :=\n  Roop.janus_inv {} {} {} {} {} {} {} {} {s} {r} h\n",
        implicit.join(" "),
        applied("entry"),
        applied("stop"),
        applied("body"),
        applied("step"),
        applied("body_inv"),
        applied("step_inv"),
        lemma_of("body", "inv_f"),
        lemma_of("step", "inv_f"),
    ));
    text.push_str(&format!(
        "theorem {f_inv} {} {{{s} {r} : {state}}} (h : {backward} {r} = Except.ok {s}) :\n    {forward} {s} = Except.ok {r} :=\n  Roop.janus_inv {} {} {} {} {} {} {} {} {r} {s} h\n",
        implicit.join(" "),
        applied("stop"),
        applied("entry"),
        applied("body_inv"),
        applied("step_inv"),
        applied("body"),
        applied("step"),
        lemma_of("body", "f_inv"),
        lemma_of("step", "f_inv"),
    ));
    (text, vec![inv_f, f_inv])
}

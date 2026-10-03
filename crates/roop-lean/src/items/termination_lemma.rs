use crate::{LemmaShape, Lifted, coded_type};

/// The theorem that a loop ends, when its state is counted: given enough fuel
/// it does not run forever, unless a piece of it runs out of fuel itself. The
/// pieces are the hypotheses, so a loop with an inner loop is not claimed to
/// end by this alone.
pub fn termination_lemma(info: &Lifted, shape: &LemmaShape) -> Option<String> {
    if !info.state.iter().all(|(_, ty)| coded_type(ty)) {
        return None;
    }
    let s = "\u{ab}__s\u{bb}";
    let parts = ["entry", "stop", "body", "step"];
    let no_fuel: Vec<String> = parts
        .iter()
        .map(|part| {
            format!(
                "(\u{ab}h{part}\u{bb} : \u{2200} \u{ab}__x\u{bb}, {} \u{ab}__x\u{bb} \u{2260} Except.error Roop.Fail.fuel)",
                shape.applied(part)
            )
        })
        .collect();
    let applied: Vec<String> = parts.iter().map(|part| shape.applied(part)).collect();
    let witnesses: Vec<String> = parts
        .iter()
        .map(|part| format!("\u{ab}h{part}\u{bb}"))
        .collect();
    Some(format!(
        "theorem {} {} {{{s} : {}}} {} (\u{ab}__e\u{bb} : {} {s} = Except.ok true) :\n    \u{2203} \u{ab}__n\u{bb}, Roop.janusGo {} \u{ab}__n\u{bb} {s} \u{2260} Except.error Roop.Fail.fuel :=\n  Roop.janus_terminates {} {} {} {} {} {s} \u{ab}__e\u{bb}\n",
        shape.theorem("loop_terminates"),
        shape.implicit,
        shape.state,
        no_fuel.join(" "),
        applied[0],
        applied.join(" "),
        applied.join(" "),
        shape.applied("body_inv"),
        shape.applied("step_inv"),
        witnesses.join(" "),
        format_args!(
            "{} {}",
            shape.theorem_applied("body_inv_f"),
            shape.theorem_applied("step_inv_f")
        ),
    ))
}

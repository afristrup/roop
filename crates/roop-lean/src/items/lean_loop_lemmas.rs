use crate::{Lifted, LemmaShape, LoopLemmas, never_fails, no_ancilla_lemma, piece_lemmas};

/// Lemmas about one loop whose pieces were lifted out of its function: each
/// body and step is undone by its inverse, and therefore the loop is, by the
/// prelude's `Roop.janus_inv`. Neither can a loop fail on an unrestored
/// ancilla unless a piece does. Returns the text and the loop lemmas, which
/// later proofs use whenever a loop's result is known. A loop with a `try`
/// inside is undone in one direction only.
pub fn lean_loop_lemmas(
    info: &Lifted,
    deps: &[String],
    earlier: &LoopLemmas,
) -> (String, LoopLemmas) {
    let shape = LemmaShape::new(info);
    let both = !info.one_way;
    let (s, r) = ("\u{ab}__s\u{bb}", "\u{ab}__r\u{bb}");
    let mut text = String::new();
    for part in ["body", "step"] {
        text.push_str(&piece_lemmas(&shape, part, both, deps, earlier));
    }

    let janus = |order: [&str; 4]| {
        let parts: Vec<String> = order.iter().map(|part| shape.applied(part)).collect();
        parts.join(" ")
    };
    let (fwd, bwd) = (
        janus(["entry", "stop", "body", "step"]),
        janus(["stop", "entry", "body_inv", "step_inv"]),
    );
    let mut chain = Vec::new();
    let mut directions = vec![("loop_inv_f", &fwd, &bwd, "inv_f", s, r)];
    if both {
        directions.push(("loop_f_inv", &bwd, &fwd, "f_inv", r, s));
    }
    for (name, from, to, suffix, x, y) in directions {
        let theorem = shape.theorem(name);
        text.push_str(&format!(
            "theorem {theorem} {} {{{s} {r} : {}}} (h : Roop.janus {from} {x} = Except.ok {y}) :\n    Roop.janus {to} {y} = Except.ok {x} :=\n  Roop.janus_inv {from} {to} {} {} {x} {y} h\n",
            shape.implicit,
            shape.state,
            shape.theorem_applied(&format!("body_{suffix}")),
            shape.theorem_applied(&format!("step_{suffix}")),
        ));
        chain.push(theorem);
    }

    let mut rewrite = Vec::new();
    for part in ["entry", "stop", "body", "step", "body_inv", "step_inv"] {
        text.push_str(&no_ancilla_lemma(&shape, part, deps, earlier));
    }
    for (name, order) in [
        ("loop_no_ancilla", ["entry", "stop", "body", "step"]),
        (
            "loop_inv_no_ancilla",
            ["stop", "entry", "body_inv", "step_inv"],
        ),
    ] {
        let theorem = shape.theorem(name);
        let witnesses: Vec<String> = order.iter().map(|part| never_fails(&shape, part)).collect();
        text.push_str(&format!(
            "theorem {theorem} {} {{{s} : {}}} :\n    Roop.janus {} {s} \u{2260} Except.error Roop.Fail.ancilla :=\n  Roop.janus_no_ancilla {} {} {s}\n",
            shape.implicit,
            shape.state,
            janus(order),
            janus(order),
            witnesses.join(" "),
        ));
        rewrite.push(theorem);
    }
    (text, LoopLemmas { chain, rewrite })
}

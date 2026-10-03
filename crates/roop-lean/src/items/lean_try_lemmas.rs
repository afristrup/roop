use crate::{HYP, LemmaShape, Lifted, LoopLemmas, never_fails, no_ancilla_lemma, piece_lemmas};

/// Lemmas about one `try` whose body and handler were lifted out of their
/// function. If each is undone by its inverse, then so is the `try`, by the
/// prelude's `Roop.tryCatch_inv`: the outcome says which one to undo. That
/// holds in one direction only, since the inverse of a handler applies to
/// states the body would not have failed on. And the `try` cannot fail on an
/// unrestored ancilla unless its handler does.
pub fn lean_try_lemmas(
    info: &Lifted,
    deps: &[String],
    earlier: &LoopLemmas,
) -> (String, LoopLemmas) {
    let shape = LemmaShape::new(info);
    let mut text = String::new();
    for part in ["body", "handler"] {
        text.push_str(&piece_lemmas(&shape, part, false, deps, earlier));
    }
    for part in ["body", "handler", "body_inv", "handler_inv"] {
        text.push_str(&no_ancilla_lemma(&shape, part, deps, earlier));
    }
    let (s, p) = ("\u{ab}__s\u{bb}", "\u{ab}__p\u{bb}");
    let forward = format!(
        "Roop.tryCatch {} {}",
        shape.applied("body"),
        shape.applied("handler")
    );
    let backward = format!(
        "Roop.untry {} {}",
        shape.applied("body_inv"),
        shape.applied("handler_inv")
    );
    let inv_f = shape.theorem("try_inv_f");
    text.push_str(&format!(
        "theorem {inv_f} {} {{{s} : {}}} {{{p} : {} \u{d7} Bool}} ({HYP} : {forward} {s} = Except.ok {p}) :\n    {backward} {p}.1 {p}.2 = Except.ok {s} :=\n  Roop.tryCatch_inv {} {} {} {} {} {} {s} {p} {HYP}\n",
        shape.implicit,
        shape.state,
        shape.state,
        shape.applied("body"),
        shape.applied("handler"),
        shape.applied("body_inv"),
        shape.applied("handler_inv"),
        shape.theorem_applied("body_inv_f"),
        shape.theorem_applied("handler_inv_f"),
    ));
    let forward_never = shape.theorem("try_no_ancilla");
    text.push_str(&format!(
        "theorem {forward_never} {} {{{s} : {}}} :\n    {forward} {s} \u{2260} Except.error Roop.Fail.ancilla :=\n  Roop.tryCatch_no_ancilla {} {} {} {s}\n",
        shape.implicit,
        shape.state,
        shape.applied("body"),
        shape.applied("handler"),
        never_fails(&shape, "handler"),
    ));
    let backward_never = shape.theorem("try_inv_no_ancilla");
    text.push_str(&format!(
        "theorem {backward_never} {} {{{s} : {}}} {{{p} : Bool}} :\n    {backward} {s} {p} \u{2260} Except.error Roop.Fail.ancilla :=\n  Roop.untry_no_ancilla {} {} {} {} {s} {p}\n",
        shape.implicit,
        shape.state,
        shape.applied("body_inv"),
        shape.applied("handler_inv"),
        never_fails(&shape, "body_inv"),
        never_fails(&shape, "handler_inv"),
    ));
    (
        text,
        LoopLemmas {
            chain: vec![inv_f],
            rewrite: vec![forward_never, backward_never],
        },
    )
}

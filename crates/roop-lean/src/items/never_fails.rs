use crate::LemmaShape;

/// `fun a h => no_ancilla_lemma caps a h`, the proof that a piece never fails
/// on an unrestored ancilla, in the shape the prelude's lemmas ask for.
pub fn never_fails(shape: &LemmaShape, part: &str) -> String {
    format!(
        "(fun \u{ab}__a\u{bb} h => {} \u{ab}__a\u{bb} h)",
        shape.theorem_applied(&format!("{part}_no_ancilla"))
    )
}

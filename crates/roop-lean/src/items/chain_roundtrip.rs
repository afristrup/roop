use crate::{HYP, LITERALS, LoopLemmas, esc_fn, esc_thm};
use std::collections::HashMap;

/// The roundtrip theorem of a function made of calls, proved from what each
/// callee guarantees. The hypothesis is about running `from` (`f` or `f_inv`):
/// its calls are peeled off in order, each leaving the result it gave, and the
/// ancilla checks at the end leave what they say. Then `to`, the function run
/// the other way, makes the same calls in reverse order, each the inverse of one
/// of those, so the callee's lemma tells what it gives, and the whole body
/// collapses to the values it started with.
pub fn chain_roundtrip(
    from: &str,
    to: &str,
    calls: &[(String, bool)],
    modular: &HashMap<String, LoopLemmas>,
) -> Option<String> {
    let mut rewrites = Vec::new();
    for position in 1..=calls.len() {
        let index = calls.len() + 1 - position;
        let (callee, inverse) = &calls[index - 1];
        let lemma = esc_thm(&format!(
            "{callee}_{}",
            if *inverse { "f_inv_c" } else { "inv_f_c" }
        ));
        if !modular.get(callee)?.chain.contains(&lemma) {
            return None;
        }
        rewrites.push(format!(
            "  try simp (disch := simp [{}]) only [Roop.guard_ok, {LITERALS}]\n  rw [{lemma} hx{index}, Roop.ok_bind]\n",
            crate::SIMP_BASE
        ));
    }
    let mut out = format!(
        "  simp only [{}] at {HYP}\n  try simp (disch := simp [{}]) only [{LITERALS}] at {HYP}\n",
        esc_fn(from),
        crate::SIMP_BASE
    );
    let guards = format!(
        "  repeat (with_reducible (obtain \u{27e8}_, hc, {HYP}\u{27e9} := Roop.bind_ok (x := Roop.check _ _) {HYP}); replace hc := beq_iff_eq.mp (Roop.check_ok hc); try subst hc)\n"
    );
    for k in 1..=calls.len() {
        out.push_str(&guards);
        out.push_str(&format!(
            "  obtain \u{27e8}z{k}, hx{k}, {HYP}\u{27e9} := Roop.bind_ok {HYP}\n"
        ));
    }
    out.push_str(&guards);
    out.push_str(&format!(
        "  have \u{ab}__e\u{bb} := Except.ok.inj {HYP}\n  subst \u{ab}__e\u{bb}\n  simp only [{}]\n  try simp (disch := simp [{}]) only [{LITERALS}]\n",
        esc_fn(to),
        crate::SIMP_BASE
    ));
    out.extend(rewrites);
    out.push_str(&format!(
        "  simp [{}, Roop.check, Roop.agetN_ok, Roop.asetN_ok, bind, Except.bind, pure, Except.pure]\n  all_goals roop_vec\n",
        crate::SIMP_BASE
    ));
    Some(out)
}

use crate::{LoopLemmas, SIMP_BASE, esc_fn, esc_thm};
use std::collections::HashMap;

/// The proof that a function made of calls never fails on an unrestored
/// ancilla, one call at a time. The first part of the body is peeled off, and
/// either an earlier call's lemma says what it gives, because it undoes that
/// call, or the callee's own lemma says it cannot fail that way. Nothing is
/// split into cases and no callee is unfolded, so it stays small however many
/// calls there are.
pub fn chain_proof(
    name: &str,
    calls: &[(String, bool)],
    modular: &HashMap<String, LoopLemmas>,
    target: &str,
) -> Option<String> {
    let has = |callee: &str, field: fn(&LoopLemmas) -> &Vec<String>, suffix: &str| {
        modular
            .get(callee)
            .is_some_and(|l| field(l).contains(&esc_thm(&format!("{callee}_{suffix}"))))
    };
    let mut out = format!("  simp only [{}] at {target}\n", esc_fn(name));
    for (k, (callee, inverse)) in calls.iter().enumerate() {
        let suffix = if *inverse {
            "inv_no_ancilla"
        } else {
            "no_ancilla"
        };
        if !has(callee, |l| &l.rewrite, suffix) {
            return None;
        }
        let mut options = Vec::new();
        for (i, (earlier, was_inverse)) in calls[..k].iter().enumerate().rev() {
            if earlier != callee || was_inverse == inverse {
                continue;
            }
            let lemma = if *was_inverse { "f_inv_c" } else { "inv_f_c" };
            if has(callee, |l| &l.chain, lemma) {
                options.push(format!(
                    "rw [{}, Roop.ok_bind] at {target}",
                    format_args!("{} hx{}", esc_thm(&format!("{callee}_{lemma}")), i + 1)
                ));
            }
        }
        options.push(format!(
            "refine (Roop.bind_ancilla {target}).elim (fun h{n} => {} h{n}) (fun \u{27e8}z{n}, hx{n}, {target}\u{27e9} => ?_)",
            esc_thm(&format!("{callee}_{suffix}")),
            n = k + 1,
        ));
        out.push_str(&format!(
            "  try simp (disch := simp [{SIMP_BASE}]) only [Roop.guard_ok] at {target}\n  first\n    | {}\n",
            options.join("\n    | ")
        ));
    }
    out.push_str(&format!(
        "  try simp (disch := simp [{SIMP_BASE}]) only [Roop.guard_ok] at {target}\n  simp [{SIMP_BASE}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at {target}\n"
    ));
    Some(out)
}

use crate::LoopLemmas;

pub const SIMP_BASE: &str = "beq_iff_eq, Prod.eta, Vector.getElem_set_self, Vector.set_set, Vector.set_getElem_self, BitVec.xor_assoc, BitVec.xor_self, BitVec.xor_zero, BitVec.add_sub_cancel, BitVec.sub_add_cancel, Roop.neg_add_self, Roop.add_neg_self, Roop.divOk_mul, Roop.sdiv_mul, Roop.mulOk_div, Roop.mul_sdiv";

/// Accesses at literal indices, and the binds of what they return, settled one
/// call at a time in the proofs by chains of calls.
pub const LITERALS: &str = "Roop.agetN_ok, Roop.asetN_ok, Roop.ok_bind, Vector.getElem_set_self, Vector.getElem_set_ne, Vector.set_set, Vector.set_getElem_self";

/// Calls in sequence feed each other's results, one loop per round. A function
/// with many calls gets a round for each, up to a limit.
pub const ROUNDS: usize = 4;
const MOST_ROUNDS: usize = 24;

/// The proof script. Split every branch (pruning the impossible ones as we go),
/// simplify the results into equations, bring in the lemma of every loop whose
/// result is known, substitute, then finish with simplification, linear
/// bit-vector arithmetic, a SAT call on the bit-blasted goal, or
/// extensionality for arrays and structs.
pub fn proof_script(loop_lemmas: &LoopLemmas, calls: usize) -> String {
    let mut simp = SIMP_BASE.to_string();
    for lemma in &loop_lemmas.rewrite {
        simp.push_str(", ");
        simp.push_str(lemma);
    }
    let loops: String = loop_lemmas
        .chain
        .iter()
        .map(|lemma| format!("  all_goals (try roop_loop {lemma})\n"))
        .collect();
    let with_ext = format!("{simp}, Prod.ext_iff");
    let round = format!(
        "  all_goals (try simp [{simp}] at *)
  all_goals (try subst_vars)
  all_goals (try roop_uncycle)
{loops}  all_goals (try (repeat roop_pairs))
  all_goals (try simp [{simp}] at *)
  all_goals (first | roop_catch (simp_all [{simp}]) | (try simp_all [{with_ext}]))
  all_goals (try subst_vars)
"
    );
    let rounds = if loop_lemmas.chain.is_empty() {
        1
    } else {
        ROUNDS.max(calls + 1).min(MOST_ROUNDS)
    };
    format!(
        "  all_goals (try (repeat' roop_cases))
  all_goals (try roop_slt)
  all_goals (try (repeat' (first | (split at *; all_goals (try simp [{simp}] at *)) | split)))
  all_goals (try simp [{simp}] at *)
  all_goals (try subst_vars)
{}  all_goals (try simp [{with_ext}] at *)
  all_goals (try (repeat' (apply And.intro)))
  all_goals (try (first | done | ((try (repeat roop_ands)); (try subst_vars); (try roop_fields); (try simp at *); ext : 1 <;> simp [*]; done) | ((try (repeat roop_ands)); (try subst_vars); roop_unfold_cyclic; simp [{simp}]; done) | ((try (repeat roop_ands)); (try subst_vars); roop_pointwise; simp [Vector.getElem_set] at *; (repeat' split) <;> (try subst_vars) <;> (try simp_all) <;> (try omega); done)))
  all_goals first | done | assumption | roop_vec | omega | bv_omega | bv_decide | (ext : 2 <;> simp [Vector.getElem_set] <;> (repeat' split) <;> (try subst_vars) <;> (try simp) <;> (try rfl) <;> (try omega))",
        round.repeat(rounds)
    )
}

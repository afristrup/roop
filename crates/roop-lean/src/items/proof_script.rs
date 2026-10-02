const LEMMAS: &str = "beq_iff_eq, Prod.eta, Vector.getElem_set_self, Vector.set_set, Vector.set_getElem_self, BitVec.xor_assoc, BitVec.xor_self, BitVec.xor_zero, BitVec.add_sub_cancel, BitVec.sub_add_cancel";

/// The proof script. Split every branch (pruning the impossible ones as we go),
/// simplify the results into equations, bring in the lemma of every loop whose
/// result is known, substitute, then finish with simplification, linear
/// bit-vector arithmetic, a SAT call on the bit-blasted goal, or
/// extensionality for arrays and structs.
pub fn proof_script(loop_lemmas: &[String]) -> String {
    let loops: String = loop_lemmas
        .iter()
        .map(|lemma| format!("  all_goals (try roop_loop {lemma})\n"))
        .collect();
    format!(
        "  all_goals (try (repeat' (first | (split at *; all_goals (try simp [{LEMMAS}] at *)) | split)))
  all_goals (try simp [{LEMMAS}] at *)
{loops}  all_goals (try subst_vars)
  all_goals (try simp_all [{LEMMAS}])
  all_goals first | done | bv_omega | bv_decide | (ext : 2 <;> simp [Vector.getElem_set] <;> (repeat' split) <;> (try subst_vars) <;> (try simp) <;> (try rfl) <;> (try omega))"
    )
}

use crate::{esc, is_mut_ref, lean_type, tuple_expr, tuple_proj, tuple_type};
use roop_syntax::{FnDef, Param};

/// The proof script. Unfold both functions, split every branch, simplify the
/// results into equations, substitute them, and finish with simplification,
/// linear bit-vector arithmetic, or a SAT call on the bit-blasted goal.
const SCRIPT: &str = "  all_goals (try (repeat' (first | split at h | split)))
  all_goals (try simp at *)
  all_goals (try subst_vars)
  all_goals (try simp_all [BitVec.xor_assoc, BitVec.xor_self, BitVec.xor_zero, BitVec.add_sub_cancel, BitVec.sub_add_cancel])
  all_goals first | done | bv_omega | bv_decide";

/// The two reversibility theorems of a function with mutable parameters:
/// running `f` and then `f_inv` returns the inputs, and `f_inv` then `f`
/// returns the outputs. Together they say no information is lost.
/// Functions with loops need an induction, so their proofs are left open.
pub fn lean_theorems(def: &FnDef, deps: &[String], loops: bool) -> Option<String> {
    let mutable: Vec<&Param> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    if mutable.is_empty() {
        return None;
    }
    let n = mutable.len();
    let tuple: String = tuple_type(&mutable.iter().map(|p| p.ty.clone()).collect::<Vec<_>>());
    let typed = |p: &Param| format!("({} : {})", esc(&p.name), lean_type(&p.ty));
    let name_of = |p: &Param| esc(&p.name);

    // Arguments of a call where every mutable parameter is the k-th component
    // of `value`; read-only parameters keep their own names.
    let with_components = |value: &str| -> Vec<String> {
        let mut k = 0;
        def.params
            .iter()
            .map(|p| {
                if is_mut_ref(&p.ty) {
                    k += 1;
                    tuple_proj(value, k - 1, n)
                } else {
                    name_of(p)
                }
            })
            .collect()
    };
    let inputs: Vec<String> = def.params.iter().map(name_of).collect();
    let initial: Vec<String> = mutable.iter().map(|p| name_of(p)).collect();
    let (f, f_inv) = (esc(&def.name), esc(&format!("{}_inv", def.name)));

    let mut unfold = vec![f.clone(), f_inv.clone()];
    unfold.extend(deps.iter().cloned());
    let proof = if loops {
        "  sorry -- open: a loop needs an induction over its iterations\n".to_string()
    } else {
        format!(
            "  simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at h \u{22a2}\n{SCRIPT}\n",
            unfold.join(", ")
        )
    };

    let all_params: Vec<String> = def.params.iter().map(typed).collect();
    let read_only: Vec<String> = def
        .params
        .iter()
        .filter(|p| !is_mut_ref(&p.ty))
        .map(typed)
        .collect();
    let mut text = format!(
        "theorem {} {} (r : {tuple}) (h : {f} {} = Except.ok r) :\n    {f_inv} {} = Except.ok {} := by\n{proof}\n",
        esc(&format!("{}_inv_f", def.name)),
        all_params.join(" "),
        inputs.join(" "),
        with_components("r").join(" "),
        tuple_expr(&initial),
    );
    text.push_str(&format!(
        "theorem {} {} (r : {tuple}) (s : {tuple}) (h : {f_inv} {} = Except.ok s) :\n    {f} {} = Except.ok r := by\n{proof}\n",
        esc(&format!("{}_f_inv", def.name)),
        read_only.join(" "),
        with_components("r").join(" "),
        with_components("s").join(" "),
    ));
    Some(text)
}

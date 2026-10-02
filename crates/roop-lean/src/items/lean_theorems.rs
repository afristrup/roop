use crate::{esc, esc_fn, is_mut_ref, lean_type, tuple_expr, tuple_proj, tuple_type};
use roop_syntax::{FnDef, Param};

/// Names of the generated result variables; double underscores keep them clear
/// of user parameters.
const OUT: &str = "\u{ab}__out\u{bb}";
const INIT: &str = "\u{ab}__init\u{bb}";

/// The proof script. Split every branch (pruning the impossible ones as we go),
/// simplify the results into equations, substitute them, then finish with
/// simplification, linear bit-vector arithmetic, a SAT call on the bit-blasted
/// goal, or extensionality for arrays and structs.
const SCRIPT: &str = "  all_goals (try (repeat' (first | (split at *; all_goals (try simp at *)) | split)))
  all_goals (try simp at *)
  all_goals (try subst_vars)
  all_goals (try simp_all [beq_iff_eq, Vector.getElem_set_self, Vector.set_set, Vector.set_getElem_self, BitVec.xor_assoc, BitVec.xor_self, BitVec.xor_zero, BitVec.add_sub_cancel, BitVec.sub_add_cancel])
  all_goals first | done | bv_omega | bv_decide | (ext : 2 <;> simp [Vector.getElem_set] <;> (repeat' split) <;> (try subst_vars) <;> (try simp) <;> (try rfl) <;> (try omega))";

const OPEN: &str = "  sorry -- open: a loop needs an induction over its iterations\n";

/// The theorems of a reversible function.
///
/// With mutable parameters: running `f` then `f_inv` returns the inputs, and
/// `f_inv` then `f` returns the outputs. Together they say no information is
/// lost. With ancillas: neither direction ever fails because an ancilla was
/// not restored; without that, a function that always failed would satisfy the
/// roundtrip theorems vacuously. Functions with loops need an induction, so
/// their proofs are left open.
pub fn lean_theorems(def: &FnDef, deps: &[String], loops: bool, ancillas: bool) -> Option<String> {
    let mutable: Vec<&Param> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    if mutable.is_empty() && !ancillas {
        return None;
    }
    let (f, f_inv) = (esc_fn(&def.name), esc_fn(&format!("{}_inv", def.name)));
    let mut unfold = vec![f.clone(), f_inv.clone()];
    unfold.extend(deps.iter().cloned());
    let simp = |target: &str| {
        format!(
            "  simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at {target}\n",
            unfold.join(", ")
        )
    };
    let proof = |target: &str| {
        if loops {
            OPEN.to_string()
        } else {
            format!("{}{SCRIPT}\n", simp(target))
        }
    };
    let typed = |p: &Param| format!("({} : {})", esc(&p.name), lean_type(&p.ty));
    let all_params: Vec<String> = def.params.iter().map(typed).collect();
    let inputs: Vec<String> = def.params.iter().map(|p| esc(&p.name)).collect();
    let thm = |suffix: &str| format!("Thm.{}", esc(&format!("{}_{suffix}", def.name)));

    let mut text = String::new();
    if ancillas {
        for (name, callee) in [("ancilla_restored", &f), ("inv_ancilla_restored", &f_inv)] {
            text.push_str(&format!(
                "theorem {} {} (h : {callee} {} = Except.error Roop.Fail.ancilla) : False := by\n{}\n",
                thm(name),
                all_params.join(" "),
                inputs.join(" "),
                proof("h")
            ));
        }
    }
    if mutable.is_empty() {
        return Some(text);
    }

    let n = mutable.len();
    let tuple: String = tuple_type(&mutable.iter().map(|p| p.ty.clone()).collect::<Vec<_>>());
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
                    esc(&p.name)
                }
            })
            .collect()
    };
    let initial: Vec<String> = mutable.iter().map(|p| esc(&p.name)).collect();
    let read_only: Vec<String> = def
        .params
        .iter()
        .filter(|p| !is_mut_ref(&p.ty))
        .map(typed)
        .collect();
    let roundtrip = proof("h \u{22a2}");
    text.push_str(&format!(
        "theorem {} {} ({OUT} : {tuple}) (h : {f} {} = Except.ok {OUT}) :\n    {f_inv} {} = Except.ok {} := by\n{roundtrip}\n",
        thm("inv_f"),
        all_params.join(" "),
        inputs.join(" "),
        with_components(OUT).join(" "),
        tuple_expr(&initial),
    ));
    text.push_str(&format!(
        "theorem {} {} ({OUT} : {tuple}) ({INIT} : {tuple}) (h : {f_inv} {} = Except.ok {INIT}) :\n    {f} {} = Except.ok {OUT} := by\n{roundtrip}\n",
        thm("f_inv"),
        read_only.join(" "),
        with_components(OUT).join(" "),
        with_components(INIT).join(" "),
    ));
    Some(text)
}

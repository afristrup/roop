use crate::{esc, is_mut_ref, lean_type, tuple_expr, tuple_type};
use roop_syntax::FnDef;

const CLOSE: &str = "first | done | (simp [BitVec.xor_assoc, BitVec.xor_self, BitVec.xor_zero, BitVec.add_sub_cancel, BitVec.sub_add_cancel]; done) | bv_omega | bv_decide";

/// The two reversibility theorems of a function with mutable parameters:
/// running `f` and then `f_inv` returns the inputs, and `f_inv` then `f`
/// returns the outputs. Together they say no information is lost.
/// `loops` functions need an induction, so their proofs are left open.
pub fn lean_theorems(def: &FnDef, deps: &[String], loops: bool) -> Option<String> {
    let mutable: Vec<_> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    if mutable.is_empty() {
        return None;
    }
    let out_name = |name: &str| esc(&format!("out_{name}"));
    let declared: Vec<String> = def
        .params
        .iter()
        .map(|p| format!("({} : {})", esc(&p.name), lean_type(&p.ty)))
        .collect();
    let outs: Vec<String> = mutable
        .iter()
        .map(|p| format!("({} : {})", out_name(&p.name), lean_type(&p.ty)))
        .collect();
    let inputs: Vec<String> = def.params.iter().map(|p| esc(&p.name)).collect();
    let after: Vec<String> = def
        .params
        .iter()
        .map(|p| {
            if is_mut_ref(&p.ty) {
                out_name(&p.name)
            } else {
                esc(&p.name)
            }
        })
        .collect();
    let in_values: Vec<String> = mutable.iter().map(|p| esc(&p.name)).collect();
    let out_values: Vec<String> = mutable.iter().map(|p| out_name(&p.name)).collect();
    let (f, f_inv) = (esc(&def.name), esc(&format!("{}_inv", def.name)));
    let mut unfold = vec![f.clone(), f_inv.clone()];
    unfold.extend(deps.iter().cloned());
    let simp = format!(
        "simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at h \u{22a2}",
        unfold.join(", ")
    );
    let proof = if loops {
        "  sorry -- open: a loop needs an induction over its iterations\n".to_string()
    } else {
        format!("  {simp}\n  <;> (try subst_vars) <;> {CLOSE}\n")
    };
    let _ = tuple_type;
    let mut text = String::new();
    for (theorem, hyp_fn, hyp_args, hyp_value, goal_fn, goal_args, goal_value) in [
        (
            "inv_f",
            &f,
            &inputs,
            &out_values,
            &f_inv,
            &after,
            &in_values,
        ),
        (
            "f_inv",
            &f_inv,
            &after,
            &in_values,
            &f,
            &inputs,
            &out_values,
        ),
    ] {
        text.push_str(&format!(
            "theorem {} {} {} (h : {} {} = Except.ok {}) :\n    {} {} = Except.ok {} := by\n{proof}\n",
            esc(&format!("{}_{theorem}", def.name)),
            declared.join(" "),
            outs.join(" "),
            hyp_fn,
            hyp_args.join(" "),
            tuple_expr(hyp_value),
            goal_fn,
            goal_args.join(" "),
            tuple_expr(goal_value),
        ));
    }
    Some(text)
}

use crate::{esc, esc_fn, esc_thm, lean_type};
use roop_syntax::FnDef;

/// Proves that a call-free function cannot report an ancilla failure.
pub fn leaf_ancilla_lemmas(def: &FnDef) -> String {
    let params: Vec<String> = def
        .params
        .iter()
        .map(|p| format!("({} : {})", esc(&p.name), lean_type(&p.ty)))
        .collect();
    let inputs: Vec<String> = def.params.iter().map(|p| esc(&p.name)).collect();
    [(def.name.clone(), "ancilla_restored"), (format!("{}_inv", def.name), "inv_ancilla_restored")]
        .into_iter()
        .map(|(name, suffix)| {
            format!(
                "theorem {} {} (h : {} {} = Except.error Roop.Fail.ancilla) : False := by\n  simp [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure] at h <;> (repeat' split at h) <;> simp_all [{}, Roop.check, Roop.aget, Roop.aset, bind, Except.bind, pure, Except.pure]\n",
                esc_thm(&format!("{}_{suffix}", def.name)),
                params.join(" "),
                esc_fn(&name),
                inputs.join(" "),
                esc_fn(&name),
                esc_fn(&name)
            )
        })
        .collect()
}

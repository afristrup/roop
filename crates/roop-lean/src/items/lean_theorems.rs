use crate::{
    HYP, LoopLemmas, chain_calls, chain_proof, chain_roundtrip, esc, esc_fn, esc_thm,
    guarded_proof, is_mut_ref, lean_type, proof_script, tuple_expr, tuple_proj, tuple_type,
    unfold_simp,
};
use roop_syntax::{FnDef, Param};
use std::collections::HashMap;

/// Names of the generated result variables; double underscores keep them clear
/// of user parameters.
const OUT: &str = "\u{ab}__out\u{bb}";
const INIT: &str = "\u{ab}__init\u{bb}";

/// The theorems of a reversible function.
///
/// With mutable parameters: running `f` then `f_inv` returns the inputs, and
/// `f_inv` then `f` returns the outputs. Together they say no information is
/// lost. With ancillas: neither direction ever fails because an ancilla was
/// not restored; without that, a function that always failed would satisfy the
/// roundtrip theorems vacuously. Loops are handled by their own lemmas. A
/// function with a `try` is undone only on states it produced, so it gets the
/// first roundtrip theorem and not the second.
pub fn lean_theorems(
    def: &FnDef,
    deps: &[String],
    loop_lemmas: &LoopLemmas,
    modular: &HashMap<String, LoopLemmas>,
    ancillas: bool,
    one_way: bool,
) -> Option<String> {
    let text = theorems(def, deps, loop_lemmas, modular, ancillas, one_way)?;
    let calls = roop_check::calls_in(&def.body, false).len();
    if calls <= 2 {
        return Some(text);
    }
    let limit = format!(
        "set_option maxHeartbeats {} in\ntheorem ",
        200_000 * calls.min(16)
    );
    Some(text.replace("theorem ", &limit))
}

fn theorems(
    def: &FnDef,
    deps: &[String],
    loop_lemmas: &LoopLemmas,
    modular: &HashMap<String, LoopLemmas>,
    ancillas: bool,
    one_way: bool,
) -> Option<String> {
    let mutable: Vec<&Param> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    if mutable.is_empty() && !ancillas {
        return None;
    }
    let (f, f_inv) = (esc_fn(&def.name), esc_fn(&format!("{}_inv", def.name)));
    let mut unfold = vec![f.clone(), f_inv.clone()];
    unfold.extend(deps.iter().cloned());
    let proof = |target: &str| {
        format!(
            "{}{}\n",
            unfold_simp(&unfold, target),
            proof_script(loop_lemmas, roop_check::calls_in(&def.body, false).len())
        )
    };
    let typed = |p: &Param| format!("({} : {})", esc(&p.name), lean_type(&p.ty));
    let all_params: Vec<String> = def.params.iter().map(typed).collect();
    let inputs: Vec<String> = def.params.iter().map(|p| esc(&p.name)).collect();
    let thm = |suffix: &str| esc_thm(&format!("{}_{suffix}", def.name));

    let mut text = String::new();
    let chain = chain_calls(def);
    let backward: Option<Vec<(String, bool)>> = chain
        .as_ref()
        .map(|calls| calls.iter().rev().map(|(c, i)| (c.clone(), !i)).collect());
    if ancillas {
        for (name, callee, raw, calls) in [
            ("ancilla_restored", &f, def.name.clone(), &chain),
            (
                "inv_ancilla_restored",
                &f_inv,
                format!("{}_inv", def.name),
                &backward,
            ),
        ] {
            let body = guarded_proof(
                calls
                    .as_ref()
                    .and_then(|calls| chain_proof(&raw, calls, modular, HYP)),
                proof(HYP),
            );
            text.push_str(&format!(
                "theorem {} {} ({HYP} : {callee} {} = Except.error Roop.Fail.ancilla) : False := by\n{body}\n",
                thm(name),
                all_params.join(" "),
                inputs.join(" "),
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
    let roundtrip = proof(&format!("{HYP} \u{22a2}"));
    let inv_f_proof = guarded_proof(
        chain.as_ref().and_then(|calls| {
            chain_roundtrip(&def.name, &format!("{}_inv", def.name), calls, modular)
        }),
        roundtrip.clone(),
    );
    let f_inv_proof = guarded_proof(
        backward.as_ref().and_then(|calls| {
            chain_roundtrip(&format!("{}_inv", def.name), &def.name, calls, modular)
        }),
        roundtrip.clone(),
    );
    text.push_str(&format!(
        "theorem {} {} ({OUT} : {tuple}) ({HYP} : {f} {} = Except.ok {OUT}) :\n    {f_inv} {} = Except.ok {} := by\n{inv_f_proof}\n",
        thm("inv_f"),
        all_params.join(" "),
        inputs.join(" "),
        with_components(OUT).join(" "),
        tuple_expr(&initial),
    ));
    if one_way {
        return Some(text);
    }
    text.push_str(&format!(
        "theorem {} {} ({OUT} : {tuple}) ({INIT} : {tuple}) ({HYP} : {f_inv} {} = Except.ok {INIT}) :\n    {f} {} = Except.ok {OUT} := by\n{f_inv_proof}\n",
        thm("f_inv"),
        read_only.join(" "),
        with_components(OUT).join(" "),
        with_components(INIT).join(" "),
    ));
    Some(text)
}

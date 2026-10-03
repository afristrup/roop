use crate::{
    HYP, LoopLemmas, esc, esc_fn, esc_thm, is_mut_ref, lean_type, tuple_expr, tuple_proj,
    tuple_type,
};
use roop_syntax::FnDef;

const OUT: &str = "\u{ab}__out\u{bb}";
const INIT: &str = "\u{ab}__init\u{bb}";

/// The theorems of a function again, in the form a caller can use: every
/// parameter implicit, so that a hypothesis about a call is enough to apply
/// them. A caller then reasons from what a callee guarantees instead of
/// unfolding its body, which keeps proofs about layers of calls small.
pub fn call_lemmas(def: &FnDef, ancillas: bool, one_way: bool) -> (String, LoopLemmas) {
    let (f, f_inv) = (esc_fn(&def.name), esc_fn(&format!("{}_inv", def.name)));
    let thm = |suffix: &str| esc_thm(&format!("{}_{suffix}", def.name));
    let mut text = String::new();
    let mut names = LoopLemmas::default();
    let implicit_all: Vec<String> = def
        .params
        .iter()
        .map(|p| format!("{{{} : {}}}", esc(&p.name), lean_type(&p.ty)))
        .collect();
    let inputs: Vec<String> = def.params.iter().map(|p| esc(&p.name)).collect();
    if ancillas {
        for (suffix, source, callee) in [
            ("no_ancilla", "ancilla_restored", &f),
            ("inv_no_ancilla", "inv_ancilla_restored", &f_inv),
        ] {
            text.push_str(&format!(
                "theorem {} {} : {callee} {} \u{2260} Except.error Roop.Fail.ancilla :=\n  fun {HYP} => {} {} {HYP}\n",
                thm(suffix),
                implicit_all.join(" "),
                inputs.join(" "),
                thm(source),
                inputs.join(" "),
            ));
            names.rewrite.push(thm(suffix));
        }
    }
    let mutable: Vec<_> = def.params.iter().filter(|p| is_mut_ref(&p.ty)).collect();
    if mutable.is_empty() {
        return (text, names);
    }
    let n = mutable.len();
    let tuple = tuple_type(&mutable.iter().map(|p| p.ty.clone()).collect::<Vec<_>>());
    let components = |value: &str| -> Vec<String> {
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
    text.push_str(&format!(
        "theorem {} {} {{{OUT} : {tuple}}} ({HYP} : {f} {} = Except.ok {OUT}) :\n    {f_inv} {} = Except.ok {} :=\n  {} {} {OUT} {HYP}\n",
        thm("inv_f_c"),
        implicit_all.join(" "),
        inputs.join(" "),
        components(OUT).join(" "),
        tuple_expr(&initial),
        thm("inv_f"),
        inputs.join(" "),
    ));
    names.chain.push(thm("inv_f_c"));
    if !one_way {
        let read_only: Vec<_> = def.params.iter().filter(|p| !is_mut_ref(&p.ty)).collect();
        let implicit_read: Vec<String> = read_only
            .iter()
            .map(|p| format!("{{{} : {}}}", esc(&p.name), lean_type(&p.ty)))
            .collect();
        let read_names: Vec<String> = read_only.iter().map(|p| esc(&p.name)).collect();
        // The outputs one by one, so a hypothesis about a call matches them.
        let outputs: Vec<String> = (1..=n).map(|k| format!("\u{ab}__o{k}\u{bb}")).collect();
        let output_binders: Vec<String> = outputs
            .iter()
            .zip(&mutable)
            .map(|(name, p)| format!("{{{name} : {}}}", lean_type(&p.ty)))
            .collect();
        let mut k = 0;
        let call_args: Vec<String> = def
            .params
            .iter()
            .map(|p| {
                if is_mut_ref(&p.ty) {
                    k += 1;
                    outputs[k - 1].clone()
                } else {
                    esc(&p.name)
                }
            })
            .collect();
        text.push_str(&format!(
            "theorem {} {} {} {{{INIT} : {tuple}}} ({HYP} : {f_inv} {} = Except.ok {INIT}) :\n    {f} {} = Except.ok {} :=\n  {} {} {} {INIT} {HYP}\n",
            thm("f_inv_c"),
            implicit_read.join(" "),
            output_binders.join(" "),
            call_args.join(" "),
            components(INIT).join(" "),
            tuple_expr(&outputs),
            thm("f_inv"),
            read_names.join(" "),
            tuple_expr(&outputs),
        ));
        names.chain.push(thm("f_inv_c"));
    }
    (text, names)
}

use crate::{
    Ctx, Dir, LeanError, LoopLemmas, PRELUDE, Translation, esc_fn, lean_commute, lean_enum,
    lean_fn, lean_loop_lemmas, lean_struct, lean_theorems,
};
use roop_check::is_irreversible_fn;
use roop_syntax::{FnDef, Item, Program};

/// Translates what it can. A function that uses something not modelled yet is
/// skipped and reported, so the rest of the file stays valid Lean.
pub fn translate(program: &Program) -> Translation {
    let mut cx = Ctx::new(program);
    let mut result = Translation::default();
    let mut lean = String::from(PRELUDE);
    lean.push('\n');
    for item in &program.items {
        if let Item::Enum(def) = item {
            lean.push_str(&lean_enum(def));
        }
    }
    for item in &program.items {
        if let Item::Struct(def) = item {
            lean.push_str(&lean_struct(def));
        }
    }

    let mut pending: Vec<&FnDef> = program
        .items
        .iter()
        .filter_map(|i| if let Item::Fn(f) = i { Some(f) } else { None })
        .collect();
    let mut deps: Vec<String> = Vec::new();
    let mut lemmas = LoopLemmas::default();
    while !pending.is_empty() {
        let mut waiting = Vec::new();
        let mut progressed = false;
        for def in pending {
            match translate_fn(&cx, def, &deps, &lemmas) {
                Ok(piece) => {
                    progressed = true;
                    lean.push_str(&piece.text);
                    cx.translated.insert(def.name.clone());
                    deps.push(esc_fn(&def.name));
                    if piece.reversible && piece.inexact {
                        deps.push(esc_fn(&format!("{}_inv", def.name)));
                        result.inexact.push(def.name.clone());
                    } else if piece.reversible {
                        deps.push(esc_fn(&format!("{}_inv", def.name)));
                        result.reversible.push(def.name.clone());
                    } else {
                        result.forward_only.push(def.name.clone());
                    }
                    if piece.parallel {
                        result.parallel.push(def.name.clone());
                    }
                    lemmas.extend(piece.lemmas);
                }
                Err(LeanError::Unknown(what)) if what.starts_with("function") => {
                    waiting.push(def);
                }
                Err(error) => {
                    progressed = true;
                    result.skipped.push((def.name.clone(), error.to_string()));
                }
            }
        }
        if !progressed {
            for def in &waiting {
                result.skipped.push((
                    def.name.clone(),
                    "depends on a function that is not modelled".into(),
                ));
            }
            break;
        }
        pending = waiting;
    }
    result.lean = lean;
    result
}

struct Piece {
    text: String,
    reversible: bool,
    inexact: bool,
    /// Whether a `#[parallel]` loop got its commutation theorem.
    parallel: bool,
    /// Lemmas about the function's loops, for the proofs that come after.
    lemmas: LoopLemmas,
}

fn translate_fn(
    cx: &Ctx,
    def: &FnDef,
    deps: &[String],
    lemmas: &LoopLemmas,
) -> Result<Piece, LeanError> {
    let forward = lean_fn(cx, def, Dir::Forward)?;
    let reversible = !is_irreversible_fn(def);
    let mut text = format!("{}{}", forward.lifted, forward.text);
    let mut known = lemmas.clone();
    let mut fresh = LoopLemmas::default();
    let mut parallel = false;
    let mut inexact = text.contains("Float");
    if reversible {
        let backward = lean_fn(cx, def, Dir::Backward)?;
        inexact |= backward.lifted.contains("Float");
        text.push_str(&backward.lifted);
        text.push_str(&backward.text);
        let ancillas = forward.ancillas > 0 || backward.ancillas > 0;
        if !inexact {
            for info in &forward.pieces {
                let (lemma_text, names) = lean_loop_lemmas(info, deps, &known);
                text.push_str(&lemma_text);
                if let Some(commute) = lean_commute(info, deps, &forward.lifted) {
                    text.push_str(&commute);
                    parallel = true;
                }
                known.extend(names.clone());
                fresh.extend(names);
            }
            if let Some(theorems) = lean_theorems(def, deps, &known, ancillas) {
                text.push_str(&theorems);
            }
        }
    }
    Ok(Piece {
        text,
        reversible,
        inexact,
        parallel,
        lemmas: fresh,
    })
}

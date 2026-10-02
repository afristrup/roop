use crate::{
    Ctx, Dir, LeanError, PRELUDE, Translation, esc, lean_enum, lean_fn, lean_struct, lean_theorems,
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
    while !pending.is_empty() {
        let mut waiting = Vec::new();
        let mut progressed = false;
        for def in pending {
            match translate_fn(&cx, def, &deps) {
                Ok(piece) => {
                    progressed = true;
                    lean.push_str(&piece.text);
                    cx.translated.insert(def.name.clone());
                    deps.push(esc(&def.name));
                    if piece.reversible && piece.inexact {
                        deps.push(esc(&format!("{}_inv", def.name)));
                        result.inexact.push(def.name.clone());
                    } else if piece.reversible {
                        deps.push(esc(&format!("{}_inv", def.name)));
                        result.reversible.push(def.name.clone());
                    } else {
                        result.forward_only.push(def.name.clone());
                    }
                    result.open.extend(piece.open);
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
    open: Vec<String>,
}

fn translate_fn(cx: &Ctx, def: &FnDef, deps: &[String]) -> Result<Piece, LeanError> {
    let (mut text, loops) = lean_fn(cx, def, Dir::Forward)?;
    let reversible = !is_irreversible_fn(def);
    let inexact = text.contains("Float");
    let mut open = Vec::new();
    if reversible {
        text.push_str(&lean_fn(cx, def, Dir::Backward)?.0);
        let theorems = if inexact {
            None
        } else {
            lean_theorems(def, deps, loops > 0)
        };
        if let Some(theorems) = theorems {
            text.push_str(&theorems);
            if loops > 0 {
                open.push(format!("{}_inv_f", def.name));
                open.push(format!("{}_f_inv", def.name));
            }
        }
    }
    Ok(Piece {
        text,
        reversible,
        open,
    })
}

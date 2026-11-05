use crate::{HYP, LemmaShape, Lifted, LoopLemmas, esc_fn, proof_script, unfold_simp};

/// A lifted piece and its inverse undo each other: `inv_f` says the inverse
/// restores what the piece produced, and with `both` so does the piece for
/// what the inverse produced.
pub fn piece_lemmas(
    info: &Lifted,
    shape: &LemmaShape,
    part: &str,
    both: bool,
    deps: &[String],
    earlier: &LoopLemmas,
) -> String {
    if part == "body" && info.additive_update.is_some() {
        return additive_update_lemmas(info, shape, both);
    }
    if part == "step" && info.scalar_additive_update.is_some() {
        return scalar_additive_update_lemmas(info, shape, both);
    }
    let (a, b) = ("\u{ab}__a\u{bb}", "\u{ab}__b\u{bb}");
    let id = &shape.id;
    let (fwd, inv) = (
        esc_fn(&format!("{id}_{part}")),
        esc_fn(&format!("{id}_{part}_inv")),
    );
    let mut unfold = vec![fwd.clone(), inv.clone()];
    unfold.extend(deps.iter().cloned());
    let directions: &[(&str, &String, &String)] = if both {
        &[("inv_f", &fwd, &inv), ("f_inv", &inv, &fwd)]
    } else {
        &[("inv_f", &fwd, &inv)]
    };
    let mut text = String::new();
    for (suffix, from, to) in directions {
        text.push_str(&format!(
            "theorem {} {} ({a} {b} : {}) ({HYP} : {from} {} {a} = Except.ok {b}) :\n    {to} {} {b} = Except.ok {a} := by\n{}{}{}\n",
            shape.theorem(&format!("{part}_{suffix}")),
            shape.explicit,
            shape.state,
            shape.args,
            shape.args,
            shape.destructure(a),
            unfold_simp(&unfold, &format!("{HYP} \u{22a2}"), false),
            proof_script(earlier, 0),
        ));
    }
    text
}

fn scalar_additive_update_lemmas(_info: &Lifted, shape: &LemmaShape, both: bool) -> String {
    let directions = if both {
        vec![
            ("inv_f", "step", "step_inv", "add_sub_cancel"),
            ("f_inv", "step_inv", "step", "sub_add_cancel"),
        ]
    } else {
        vec![("inv_f", "step", "step_inv", "add_sub_cancel")]
    };
    directions
        .into_iter()
        .map(|(suffix, from, to, cancel)| {
            format!(
                "theorem {} {} (a b : {}) ({HYP} : {} a = Except.ok b) :\n    {} b = Except.ok a := by\n  obtain ⟨a1, a2⟩ := a\n  simp [{}, {}, pure, Except.pure, BitVec.{}] at {HYP} ⊢\n  subst b\n  simp [BitVec.{}]\n",
                shape.theorem(&format!("step_{suffix}")),
                shape.explicit,
                shape.state,
                shape.applied(from),
                shape.applied(to),
                shape.piece(from),
                shape.piece(to),
                cancel,
                cancel,
            )
        })
        .collect()
}

fn additive_update_lemmas(info: &Lifted, shape: &LemmaShape, both: bool) -> String {
    let update = info.additive_update.as_ref().unwrap();
    let directions = if both {
        vec![
            (
                "inv_f",
                "body",
                "body_inv",
                "aget_aset_add_sub_inverse_state",
                "+",
            ),
            (
                "f_inv",
                "body_inv",
                "body",
                "aget_aset_sub_add_inverse_state",
                "-",
            ),
        ]
    } else {
        vec![(
            "inv_f",
            "body",
            "body_inv",
            "aget_aset_add_sub_inverse_state",
            "+",
        )]
    };
    let mut text = String::new();
    for (suffix, from, to, theorem, op) in directions {
        let delta = format!("({} {})", update.delta, shape.args);
        let (theorem, index, args) = if update.index_is_state {
            let theorem = match theorem {
                "aget_aset_add_sub_inverse_state" => "aget_aset_add_sub_inverse_by",
                "aget_aset_sub_add_inverse_state" => "aget_aset_sub_add_inverse_by",
                _ => unreachable!(),
            };
            (
                theorem,
                "a2".to_string(),
                format!("a1 (fun z => z) {delta} a2 b2 b1 __forward"),
            )
        } else {
            (
                theorem,
                update.index.clone(),
                format!("a1 {} {delta} a2 b2 b1 __forward", update.index),
            )
        };
        text.push_str(&format!(
            "theorem {} {} (a b : {}) ({HYP} : {} a = Except.ok b) :\n    {} b = Except.ok a := by\n  obtain \u{27e8}a1, a2\u{27e9} := a\n  obtain \u{27e8}b1, b2\u{27e9} := b\n  have __forward : (do\n    let x \u{2190} Roop.aget a1 {index}\n    let d \u{2190} {delta} a2\n    let out \u{2190} Roop.aset a1 {index} (x {op} d)\n    pure (out, a2)) = Except.ok (b1, b2) := by\n    simpa only [{}] using {HYP}\n  have result := Roop.{theorem} {args}\n  simpa only [{}] using result\n",
            shape.theorem(&format!("body_{suffix}")),
            shape.explicit,
            shape.state,
            shape.applied(from),
            shape.applied(to),
            shape.piece(from),
            shape.piece(to),
        ));
    }
    text
}

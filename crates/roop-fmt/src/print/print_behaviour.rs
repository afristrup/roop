use crate::Doc;
use roop_syntax::{Behaviour, ChoiceKind};

pub fn print_behaviour(behaviour: &Behaviour) -> Doc {
    match behaviour {
        Behaviour::End => Doc::text("end"),
        Behaviour::Var(name) => Doc::text(name),
        Behaviour::Rec(name, body) => Doc::group(Doc::concat(vec![
            Doc::text(format!("rec {name} {{")),
            Doc::nest(Doc::concat(vec![Doc::Line, print_behaviour(body)])),
            Doc::Line,
            Doc::text("}"),
        ])),
        Behaviour::Choice {
            kind,
            checkpoint,
            branches,
        } => {
            let word = match kind {
                ChoiceKind::Offer => "offer",
                ChoiceKind::Select => "select",
            };
            let mark = if *checkpoint { "checkpoint " } else { "" };
            let branches = branches
                .iter()
                .map(|(label, next)| {
                    Doc::concat(vec![Doc::text(format!("{label}: ")), print_behaviour(next)])
                })
                .collect();
            let sep = Doc::concat(vec![Doc::text(","), Doc::Line]);
            Doc::group(Doc::concat(vec![
                Doc::text(format!("{mark}{word} {{")),
                Doc::nest(Doc::concat(vec![
                    Doc::Line,
                    Doc::join(branches, sep),
                    Doc::IfBreak(","),
                ])),
                Doc::Line,
                Doc::text("}"),
            ]))
        }
    }
}

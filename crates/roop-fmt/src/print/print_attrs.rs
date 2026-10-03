use crate::Doc;
use roop_syntax::{Attr, Target};

/// Each attribute on a line of its own, above the statement.
pub fn print_attrs(attrs: &[Attr]) -> Vec<Doc> {
    attrs
        .iter()
        .flat_map(|attr| {
            let text = match attr {
                Attr::Concurrent => "#[concurrent]".to_string(),
                Attr::Parallel { target: None } => "#[parallel]".to_string(),
                Attr::Parallel { target: Some(t) } => {
                    let name = match t {
                        Target::Cpu => "cpu",
                        Target::Nvptx => "nvptx",
                        Target::Metal => "metal",
                    };
                    format!("#[parallel({name})]")
                }
            };
            [Doc::text(text), Doc::HardLine]
        })
        .collect()
}

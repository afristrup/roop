use crate::{Lifted, esc, esc_fn, esc_thm, lean_type, tuple_type, unref};

/// The pieces of text the lemmas about a lifted loop or `try` share.
pub struct LemmaShape {
    pub id: String,
    pub args: String,
    /// The captured variables as explicit binders, `(x : T)`.
    pub explicit: String,
    /// The captured variables as implicit binders, `{x : T}`.
    pub implicit: String,
    /// The type of the state tuple.
    pub state: String,
    components: usize,
}

impl LemmaShape {
    pub fn new(info: &Lifted) -> Self {
        let binders = |open: &str, close: &str| {
            info.captures
                .iter()
                .map(|(n, t)| format!("{open}{} : {}{close}", esc(n), lean_type(t)))
                .collect::<Vec<_>>()
                .join(" ")
        };
        LemmaShape {
            id: info.id.clone(),
            args: info
                .captures
                .iter()
                .map(|(n, _)| esc(n))
                .collect::<Vec<_>>()
                .join(" "),
            explicit: binders("(", ")"),
            implicit: binders("{", "}"),
            state: tuple_type(&info.state.iter().map(|(_, t)| unref(t)).collect::<Vec<_>>()),
            components: info.state.len(),
        }
    }

    /// A lifted definition applied to the captured variables.
    pub fn applied(&self, part: &str) -> String {
        format!("({} {})", esc_fn(&format!("{}_{part}", self.id)), self.args)
    }

    /// The bare name of a lifted definition.
    pub fn piece(&self, part: &str) -> String {
        esc_fn(&format!("{}_{part}", self.id))
    }

    /// The name of a theorem about this construct.
    pub fn theorem(&self, name: &str) -> String {
        esc_thm(&format!("{}_{name}", self.id))
    }

    /// A theorem applied to the captured variables.
    pub fn theorem_applied(&self, name: &str) -> String {
        format!("({} {})", self.theorem(name), self.args)
    }

    /// Opens a state variable into its components, when there are several.
    pub fn destructure(&self, var: &str) -> String {
        match self.components {
            0 | 1 => String::new(),
            n => {
                let parts: Vec<String> = (1..=n)
                    .map(|k| format!("\u{ab}{}{k}\u{bb}", var.trim_matches(['\u{ab}', '\u{bb}'])))
                    .collect();
                format!("  obtain \u{27e8}{}\u{27e9} := {var}\n", parts.join(", "))
            }
        }
    }
}

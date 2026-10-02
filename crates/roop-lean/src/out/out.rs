/// Indented text being written, plus a counter for fresh names.
#[derive(Default)]
pub struct Out {
    pub text: String,
    pub indent: usize,
    /// Loops translated so far; their reversibility needs an induction.
    pub loops: usize,
    /// Ancillas whose restoration is checked, forward or backward.
    pub ancillas: usize,
    next: usize,
}

impl Out {
    pub fn line(&mut self, line: &str) {
        self.text.push_str(&"  ".repeat(self.indent));
        self.text.push_str(line);
        self.text.push('\n');
    }

    pub fn fresh(&mut self, prefix: &str) -> String {
        self.next += 1;
        format!("{prefix}{}", self.next)
    }
}

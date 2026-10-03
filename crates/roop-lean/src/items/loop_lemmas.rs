/// What later proofs can use about the loops translated so far.
#[derive(Clone, Default)]
pub struct LoopLemmas {
    /// Lemmas `roop_loop` applies to every hypothesis about a loop's result.
    pub chain: Vec<String>,
    /// Facts `simp` rewrites with.
    pub rewrite: Vec<String>,
}

impl LoopLemmas {
    pub fn extend(&mut self, other: LoopLemmas) {
        self.chain.extend(other.chain);
        self.rewrite.extend(other.rewrite);
    }
}

use roop_syntax::Stmt;

/// A parallel loop absorbing its neighbours. Each absorbed loop's induction
/// variable stays at `lo` during the fused loop, so it is asserted before
/// and advanced to `hi` after, exactly as the unfused loop would leave it.
pub struct Fused {
    pub head: Stmt,
    pub prologue: Vec<Stmt>,
    pub epilogue: Vec<Stmt>,
}
